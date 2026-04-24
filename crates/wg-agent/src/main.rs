#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Parser;
use tokio::sync::{broadcast, mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use tracing::{error, info, warn};
use wg_shared::types::Feature;

mod backoff;
mod capabilities;
mod config;
mod failure_buffer;
mod panic_hook;
mod platform;
mod state;

use backoff::Backoff;
use config::Config;
use failure_buffer::{FailureBuffer, FailureEntry};
use state::{DaemonState, IdleReason};

// ---------------------------------------------------------------------------
// Proto includes
// ---------------------------------------------------------------------------

mod proto {
    // wallguard.control.rs uses `super::models::*`
    pub mod models {
        tonic::include_proto!("wallguard.models");
    }
    pub mod control {
        tonic::include_proto!("wallguard.control");
    }
    // wallguard.cli.rs uses `super::control::MonitoringStatus`
    pub mod cli {
        tonic::include_proto!("wallguard.cli");
    }
}

use proto::control::{
    client_message, control_client::ControlClient, server_message, AgentFailure as ProtoFailure,
    ClientMessage, CommandResult, CommandStatus, Feature as ProtoFeature,
    FirewallKind as ProtoFirewallKind, Heartbeat, HeartbeatAck, Hello, MonitoringStatus,
    ServerMessage,
};
use proto::cli::{
    agent_control_server::{AgentControl, AgentControlServer},
    GracefulRestartRequest, GracefulRestartResponse, StatusRequest, StatusResponse,
};

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(name = "wg-agent", about = "WallGuard device agent")]
struct Args {
    /// Path to the agent config file.
    #[arg(long, default_value = "/etc/wallguard/config.toml")]
    config: std::path::PathBuf,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args = Args::parse();

    let config = match Config::load(&args.config) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("error: {e}\nRun `wg-cli enroll` to create the configuration file.");
            std::process::exit(1);
        }
    };

    // Init failure buffer before the async runtime so the panic hook can use it.
    let buf_dir = config
        .transmission
        .disk_buffer_path
        .parent()
        .unwrap_or(std::path::Path::new("/var/lib/wallguard"));
    failure_buffer::BUFFER.get_or_init(|| {
        FailureBuffer::load_or_create(buf_dir.join("failures.jsonl"))
    });

    panic_hook::install();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    if let Err(e) = rt.block_on(run(config)) {
        error!("agent exited with error: {e:#}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Async entry
// ---------------------------------------------------------------------------

async fn run(config: Arc<Config>) -> anyhow::Result<()> {
    init_tracing(&config);
    info!(
        version = env!("CARGO_PKG_VERSION"),
        os      = ?platform::TARGET_OS,
        "wg-agent starting"
    );

    let rd_available = capabilities::probe_remote_desktop().await;
    let features     = wg_shared::capabilities::derive_capabilities(
        config.device.firewall_kind,
        rd_available,
    );

    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let (state_tx, state_rx) = watch::channel(DaemonState::Provisioning);

    // Signal handler task — converts SIGTERM / SIGINT into the shutdown channel.
    let sig_tx = shutdown_tx.clone();
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())
                .expect("failed to register SIGTERM handler");
            tokio::select! {
                _ = sigterm.recv()            => info!("SIGTERM received"),
                _ = tokio::signal::ctrl_c()   => info!("SIGINT received"),
            }
        }
        #[cfg(not(unix))]
        { let _ = tokio::signal::ctrl_c().await; }

        let _ = sig_tx.send(());
    });

    // CLI gRPC server on Unix socket — background task.
    let cli_handle = {
        let cfg = config.clone();
        let rx  = state_rx.clone();
        let tx  = shutdown_tx.clone();
        tokio::spawn(async move { run_cli_server(cfg, rx, tx).await })
    };

    run_state_machine(config, features, state_tx, shutdown_tx.subscribe()).await?;

    // Stop CLI server.
    let _ = shutdown_tx.send(());
    cli_handle.abort();
    let _ = cli_handle.await;

    info!("wg-agent stopped");
    Ok(())
}

fn init_tracing(config: &Config) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if config.observability.log_format == "json" {
        fmt().json().with_env_filter(filter).init();
    } else {
        fmt().with_env_filter(filter).init();
    }
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

async fn run_state_machine(
    config:      Arc<Config>,
    features:    Vec<Feature>,
    state_tx:    watch::Sender<DaemonState>,
    mut shutdown: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    let buf     = failure_buffer::BUFFER.get().expect("buffer must be initialised");
    let mut bo  = Backoff::new(
        config.agent.reconnect_base_s as f64,
        config.agent.reconnect_max_s  as f64,
        2.0,
    );
    let mut state = initial_state(&config);

    loop {
        state_tx.send(state.clone()).ok();

        let next = tokio::select! {
            biased;
            _ = shutdown.recv() => {
                info!("shutdown signal — stopping state machine");
                break;
            }
            next = step(&state, &config, &features, buf, &mut bo) => next,
        };

        state = next;
    }

    Ok(())
}

fn initial_state(config: &Config) -> DaemonState {
    if config.tls.device_cert.exists() && config.tls.device_key.exists() {
        DaemonState::Connecting
    } else {
        DaemonState::Provisioning
    }
}

async fn step(
    state:    &DaemonState,
    config:   &Arc<Config>,
    features: &[Feature],
    buf:      &'static FailureBuffer,
    bo:       &mut Backoff,
) -> DaemonState {
    match state {
        DaemonState::Provisioning    => step_provisioning(config).await,
        DaemonState::Idle(reason)    => step_idle(reason).await,
        DaemonState::Connecting      => step_connecting(config, features, buf, bo).await,
        DaemonState::Connected { .. } => {
            // Connected state is entered and exited within step_connecting;
            // if we somehow land here, go back to Connecting.
            DaemonState::Connecting
        }
    }
}

async fn step_provisioning(config: &Config) -> DaemonState {
    tokio::time::sleep(Duration::from_secs(5)).await;
    if config.tls.device_cert.exists() && config.tls.device_key.exists() {
        info!("device certificate found — connecting");
        DaemonState::Connecting
    } else {
        DaemonState::Provisioning
    }
}

async fn step_idle(reason: &IdleReason) -> DaemonState {
    match reason {
        IdleReason::VersionRejected { min_required } => {
            warn!(
                min_required,
                "server requires protocol version {min_required}; \
                 manual agent upgrade required — agent will not retry"
            );
        }
    }
    // Park indefinitely; only the outer shutdown select exits this.
    std::future::pending::<DaemonState>().await
}

async fn step_connecting(
    config:   &Arc<Config>,
    features: &[Feature],
    buf:      &'static FailureBuffer,
    bo:       &mut Backoff,
) -> DaemonState {
    info!(server = %config.server.name, "connecting …");

    match try_connect(config, features).await {
        Err(e) => {
            let delay = bo.next();
            warn!("connection failed: {e:#} — retry in {delay:?}");
            tokio::time::sleep(delay).await;
            DaemonState::Connecting
        }
        Ok(ConnectResult::VersionRejected(min)) => {
            DaemonState::Idle(IdleReason::VersionRejected { min_required: min })
        }
        Ok(ConnectResult::Connected(cs)) => {
            bo.reset();
            info!("connected; running connected loop");
            run_connected_loop(cs, config, buf).await
        }
    }
}

// ---------------------------------------------------------------------------
// gRPC connection + handshake
// ---------------------------------------------------------------------------

struct ConnectSuccess {
    negotiated_features: Vec<Feature>,
    out_tx:              mpsc::Sender<ClientMessage>,
    in_stream:           tonic::Streaming<ServerMessage>,
}

enum ConnectResult {
    Connected(ConnectSuccess),
    VersionRejected(u32),
}

async fn try_connect(
    config:   &Config,
    features: &[Feature],
) -> anyhow::Result<ConnectResult> {
    let cert = std::fs::read_to_string(&config.tls.device_cert)?;
    let key  = std::fs::read_to_string(&config.tls.device_key)?;
    let ca   = std::fs::read_to_string(&config.tls.ca_cert)?;

    let tls = ClientTlsConfig::new()
        .domain_name(&config.server.name)
        .identity(Identity::from_pem(&cert, &key))
        .ca_certificate(Certificate::from_pem(ca));

    let channel = Channel::from_shared(config.grpc_endpoint())?
        .tls_config(tls)?
        .connect_timeout(Duration::from_secs(10))
        .connect()
        .await?;

    let mut client = ControlClient::new(channel);

    let (out_tx, out_rx) = mpsc::channel::<ClientMessage>(64);
    let out_stream       = ReceiverStream::new(out_rx);

    let response  = client.channel(out_stream).await?;
    let mut in_st = response.into_inner();

    // Send Hello.
    out_tx.send(make_hello(features, config)).await?;

    // Receive Welcome or VersionRejected.
    let msg = in_st
        .message()
        .await?
        .ok_or_else(|| anyhow::anyhow!("server closed stream before handshake complete"))?;

    match msg.message {
        Some(server_message::Message::Welcome(w)) => {
            let negotiated_features = w
                .negotiated_features
                .iter()
                .filter_map(|&f| proto_to_shared_feature(f))
                .collect::<Vec<_>>();
            info!(
                features = ?negotiated_features,
                "handshake complete"
            );
            Ok(ConnectResult::Connected(ConnectSuccess {
                negotiated_features,
                out_tx,
                in_stream: in_st,
            }))
        }
        Some(server_message::Message::VersionRejected(v)) => {
            warn!(
                min_required = v.min_required_version,
                "{}",
                v.message
            );
            Ok(ConnectResult::VersionRejected(v.min_required_version))
        }
        other => Err(anyhow::anyhow!("unexpected handshake message: {other:?}")),
    }
}

// ---------------------------------------------------------------------------
// Connected loop
// ---------------------------------------------------------------------------

async fn run_connected_loop(
    cs:     ConnectSuccess,
    config: &Config,
    buf:    &'static FailureBuffer,
) -> DaemonState {
    let ConnectSuccess { out_tx, mut in_stream, .. } = cs;

    // Replay failures buffered while disconnected.
    replay_failures(&out_tx, buf).await;

    let hb_interval  = Duration::from_secs(config.agent.heartbeat_interval_s);
    let mut hb_timer = tokio::time::interval(hb_interval);
    let mut hb_seq   = 0u64;
    let mut in_flight: HashSet<u64> = HashSet::new();

    loop {
        tokio::select! {
            _ = hb_timer.tick() => {
                if in_flight.len() >= 3 {
                    warn!("3 consecutive heartbeat acks missed — reconnecting");
                    return DaemonState::Connecting;
                }
                hb_seq += 1;
                in_flight.insert(hb_seq);
                let msg = ClientMessage {
                    message: Some(client_message::Message::Heartbeat(Heartbeat {
                        seq:               hb_seq,
                        sent_at_unix_ms:   unix_ms_now(),
                        monitoring_status: Some(MonitoringStatus::default()),
                    })),
                };
                if out_tx.send(msg).await.is_err() {
                    warn!("output stream closed — reconnecting");
                    return DaemonState::Connecting;
                }
            }

            result = in_stream.message() => {
                match result {
                    Err(e) => {
                        warn!("stream error: {e} — reconnecting");
                        return DaemonState::Connecting;
                    }
                    Ok(None) => {
                        info!("server closed stream — reconnecting");
                        return DaemonState::Connecting;
                    }
                    Ok(Some(msg)) => {
                        if !handle_server_msg(msg, &out_tx, &mut in_flight).await {
                            return DaemonState::Connecting;
                        }
                    }
                }
            }
        }
    }

}

/// Returns `false` if the caller should reconnect immediately.
async fn handle_server_msg(
    msg:       ServerMessage,
    out_tx:    &mpsc::Sender<ClientMessage>,
    in_flight: &mut HashSet<u64>,
) -> bool {
    use server_message::Message as M;

    match msg.message {
        // ── Heartbeat ────────────────────────────────────────────────────────
        Some(M::HeartbeatAck(ack)) => {
            in_flight.remove(&ack.ack_seq);
        }
        Some(M::ServerHeartbeat(hb)) => {
            let _ = out_tx.send(ClientMessage {
                message: Some(client_message::Message::HeartbeatAck(HeartbeatAck {
                    ack_seq:         hb.seq,
                    acked_at_unix_ms: unix_ms_now(),
                })),
            }).await;
        }

        // ── Graceful server shutdown ──────────────────────────────────────────
        Some(M::ShutdownImminent(s)) => {
            let delay = Duration::from_millis(s.reconnect_after_ms as u64);
            info!("server shutting down; waiting {delay:?} before reconnecting");
            tokio::time::sleep(delay).await;
            return false;
        }

        // ── Monitoring control (Phase 7 stub) ─────────────────────────────────
        Some(M::SetMonitoring(cmd)) => {
            info!(command_id = %cmd.command_id, "set_monitoring (Phase 7 stub)");
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Success, "")).await;
        }
        Some(M::ThrottleMonitoring(_)) => {
            // No response expected; monitoring rate updated in Phase 7.
        }

        // ── Tunnels (Phase 8 stubs) ───────────────────────────────────────────
        Some(M::OpenSshTunnel(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "SSH tunnel not yet available (Phase 8)")).await;
        }
        Some(M::OpenTtyTunnel(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "TTY tunnel not yet available (Phase 8)")).await;
        }
        Some(M::OpenHttpTunnel(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "HTTP tunnel not yet available (Phase 8)")).await;
        }
        Some(M::OpenRemoteDesktopTunnel(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "remote desktop not yet available (Phase 8)")).await;
        }

        // ── Firewall commands (Phase 12 stubs) ───────────────────────────────
        Some(M::CreateFilterRule(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "firewall config not yet available (Phase 12)")).await;
        }
        Some(M::CreateNatRule(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "firewall config not yet available (Phase 12)")).await;
        }
        Some(M::CreateAlias(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "firewall config not yet available (Phase 12)")).await;
        }
        Some(M::DeleteRule(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "firewall config not yet available (Phase 12)")).await;
        }
        Some(M::ApplyRuleSet(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "firewall config not yet available (Phase 12)")).await;
        }
        Some(M::ExecuteNamedCommand(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "named commands not yet available (Phase 12)")).await;
        }
        Some(M::RequestConfigSnapshot(cmd)) => {
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Failure,
                "config snapshots not yet available (Phase 12)")).await;
        }

        // ── Cert renewal (Phase 11 stub) ──────────────────────────────────────
        Some(M::RenewCertificateRequest(_)) => {
            warn!("cert renewal requested (Phase 11 stub) — ignoring");
        }

        // ── Already handled at handshake; should not appear again ─────────────
        Some(M::Welcome(_)) | Some(M::VersionRejected(_)) => {
            warn!("unexpected post-handshake message ignored");
        }

        None => {}
    }

    true
}

// ---------------------------------------------------------------------------
// Failure replay
// ---------------------------------------------------------------------------

async fn replay_failures(out_tx: &mpsc::Sender<ClientMessage>, buf: &FailureBuffer) {
    let entries = buf.read_all();
    if entries.is_empty() {
        return;
    }
    info!("replaying {} buffered failure(s)", entries.len());
    let mut delivered = Vec::new();

    for entry in &entries {
        let proto = failure_entry_to_proto(entry, true);
        let msg   = ClientMessage {
            message: Some(client_message::Message::AgentFailure(proto)),
        };
        if out_tx.send(msg).await.is_err() {
            break;
        }
        delivered.push(entry.failure_id);
    }

    buf.trim_delivered(&delivered);
    info!("replayed {} failure(s)", delivered.len());
}

// ---------------------------------------------------------------------------
// Message helpers
// ---------------------------------------------------------------------------

fn make_hello(features: &[Feature], config: &Config) -> ClientMessage {
    ClientMessage {
        message: Some(client_message::Message::Hello(Hello {
            protocol_version:       wg_shared::capabilities::PROTOCOL_VERSION,
            min_compatible_version: wg_shared::capabilities::MIN_AGENT_PROTOCOL_VERSION,
            supported_features:     features.iter().map(|&f| shared_to_proto_feature(f)).collect(),
            agent_version:          env!("CARGO_PKG_VERSION").to_string(),
            firewall_kind:          firewall_to_proto(config.device.firewall_kind),
        })),
    }
}

fn cmd_result(command_id: &str, status: CommandStatus, error_msg: &str) -> ClientMessage {
    ClientMessage {
        message: Some(client_message::Message::CommandResult(CommandResult {
            command_id:        command_id.to_string(),
            status:            status as i32,
            error_message:     error_msg.to_string(),
            applied_digest:    String::new(),
            output:            String::new(),
            applied_at_unix_ms: unix_ms_now(),
        })),
    }
}

fn failure_entry_to_proto(e: &FailureEntry, is_replay: bool) -> ProtoFailure {
    ProtoFailure {
        failure_id:  e.failure_id.to_string(),
        severity:    severity_to_proto(e.severity),
        category:    category_to_proto(e.category),
        message:     e.message.clone(),
        context:     e.context.clone().unwrap_or_default(),
        occurred_at: e.occurred_at,
        is_replay,
    }
}

// ---------------------------------------------------------------------------
// Enum conversions
// ---------------------------------------------------------------------------

fn shared_to_proto_feature(f: Feature) -> i32 {
    match f {
        Feature::NetworkMonitoring   => ProtoFeature::NetworkMonitoring as i32,
        Feature::TelemetryMonitoring => ProtoFeature::TelemetryMonitoring as i32,
        Feature::ConfigMonitoring    => ProtoFeature::ConfigMonitoring as i32,
        Feature::SshTunnel           => ProtoFeature::SshTunnel as i32,
        Feature::TtyTunnel           => ProtoFeature::TtyTunnel as i32,
        Feature::HttpTunnel          => ProtoFeature::HttpTunnel as i32,
        Feature::NamedCommands       => ProtoFeature::NamedCommands as i32,
        Feature::RemoteDesktop       => ProtoFeature::RemoteDesktop as i32,
    }
}

fn proto_to_shared_feature(i: i32) -> Option<Feature> {
    match i {
        0 => Some(Feature::NetworkMonitoring),
        1 => Some(Feature::TelemetryMonitoring),
        2 => Some(Feature::ConfigMonitoring),
        3 => Some(Feature::SshTunnel),
        4 => Some(Feature::TtyTunnel),
        5 => Some(Feature::HttpTunnel),
        6 => Some(Feature::NamedCommands),
        7 => Some(Feature::RemoteDesktop),
        _ => None,
    }
}

fn firewall_to_proto(k: wg_shared::types::FirewallKind) -> i32 {
    use wg_shared::types::FirewallKind;
    match k {
        FirewallKind::None     => ProtoFirewallKind::None as i32,
        FirewallKind::PfSense  => ProtoFirewallKind::Pfsense as i32,
        FirewallKind::OPNSense => ProtoFirewallKind::Opnsense as i32,
        FirewallKind::NFTables => ProtoFirewallKind::Nftables as i32,
    }
}

fn severity_to_proto(s: wg_shared::types::FailureSeverity) -> i32 {
    use wg_shared::types::FailureSeverity;
    match s {
        FailureSeverity::Warning => 0,
        FailureSeverity::Error   => 1,
        FailureSeverity::Fatal   => 2,
    }
}

fn category_to_proto(c: wg_shared::types::FailureCategory) -> i32 {
    use wg_shared::types::FailureCategory;
    match c {
        FailureCategory::Monitoring   => 0,
        FailureCategory::Tunnel       => 1,
        FailureCategory::DiskBuffer   => 2,
        FailureCategory::Fireparse    => 3,
        FailureCategory::AgentCrash   => 4,
        FailureCategory::Connectivity => 5,
        FailureCategory::System       => 6,
    }
}

fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// CLI gRPC server (Unix domain socket)
// ---------------------------------------------------------------------------

async fn run_cli_server(
    config:      Arc<Config>,
    state_rx:    watch::Receiver<DaemonState>,
    shutdown_tx: broadcast::Sender<()>,
) {
    use std::os::unix::fs::PermissionsExt;
    use tokio::net::UnixListener;
    use tokio_stream::wrappers::UnixListenerStream;

    let sock_path = Config::cli_socket_path();

    // Remove stale socket from a previous run.
    let _ = std::fs::remove_file(sock_path);
    if let Some(parent) = std::path::Path::new(sock_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            warn!("could not create socket dir {}: {e}", parent.display());
            return;
        }
    }

    let listener = match UnixListener::bind(sock_path) {
        Ok(l)  => l,
        Err(e) => {
            warn!("could not bind CLI socket {sock_path}: {e}");
            return;
        }
    };

    // Restrict socket to root only.
    if let Err(e) = std::fs::set_permissions(
        sock_path,
        std::fs::Permissions::from_mode(0o600),
    ) {
        warn!("could not set socket permissions: {e}");
    }

    info!("CLI socket listening on {sock_path}");

    let svc = CliServer {
        state_rx,
        config,
        start_time:  Instant::now(),
        shutdown_tx,
    };

    let mut shutdown_rx = svc.shutdown_tx.subscribe();

    let router = tonic::transport::Server::builder()
        .add_service(AgentControlServer::new(svc));

    let incoming = UnixListenerStream::new(listener);

    tokio::select! {
        result = router.serve_with_incoming(incoming) => {
            if let Err(e) = result {
                warn!("CLI server error: {e}");
            }
        }
        _ = shutdown_rx.recv() => {
            info!("CLI server shutting down");
        }
    }
}

// ---------------------------------------------------------------------------
// AgentControl implementation
// ---------------------------------------------------------------------------

struct CliServer {
    state_rx:    watch::Receiver<DaemonState>,
    config:      Arc<Config>,
    start_time:  Instant,
    shutdown_tx: broadcast::Sender<()>,
}

#[tonic::async_trait]
impl AgentControl for CliServer {
    async fn status(
        &self,
        _req: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let state = self.state_rx.borrow().clone();
        Ok(tonic::Response::new(StatusResponse {
            state:             state.to_proto_i32(),
            device_id:         self.config.device.id.clone(),
            server_url:        self.config.grpc_endpoint(),
            agent_version:     env!("CARGO_PKG_VERSION").to_string(),
            monitoring_status: Some(proto::control::MonitoringStatus::default()),
            uptime_secs:       self.start_time.elapsed().as_secs(),
        }))
    }

    async fn graceful_restart(
        &self,
        req: tonic::Request<GracefulRestartRequest>,
    ) -> Result<tonic::Response<GracefulRestartResponse>, tonic::Status> {
        let timeout_ms = req.into_inner().drain_timeout_ms;
        info!(timeout_ms, "graceful restart requested via CLI");
        let _ = self.shutdown_tx.send(());
        Ok(tonic::Response::new(GracefulRestartResponse {
            accepted: true,
            message:  "shutdown initiated".to_string(),
        }))
    }
}
