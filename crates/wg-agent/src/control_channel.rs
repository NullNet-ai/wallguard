use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use tracing::{info, warn};
use wg_shared::types::Feature;

use crate::config::Config;
use crate::disk_buffer::DiskBuffer;
use crate::failure_buffer::FailureBuffer;
use crate::proto::control::{
    client_message, control_client::ControlClient, server_message, ClientMessage, CommandStatus,
    Heartbeat, HeartbeatAck, MonitoringStatus, ServerMessage,
};
use crate::proto_conv::{
    cmd_result, failure_entry_to_proto, make_hello, proto_to_shared_feature, unix_ms_now,
};
use crate::state::DaemonState;
use crate::tunnel;

pub struct ConnectSuccess {
    pub negotiated_features: Vec<Feature>,
    pub out_tx:              mpsc::Sender<ClientMessage>,
    pub in_stream:           tonic::Streaming<ServerMessage>,
}

pub enum ConnectResult {
    Connected(ConnectSuccess),
    VersionRejected(u32),
}

pub async fn try_connect(
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

    out_tx.send(make_hello(features, config)).await?;

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
            info!(features = ?negotiated_features, "handshake complete");
            Ok(ConnectResult::Connected(ConnectSuccess {
                negotiated_features,
                out_tx,
                in_stream: in_st,
            }))
        }
        Some(server_message::Message::VersionRejected(v)) => {
            warn!(min_required = v.min_required_version, "{}", v.message);
            Ok(ConnectResult::VersionRejected(v.min_required_version))
        }
        other => Err(anyhow::anyhow!("unexpected handshake message: {other:?}")),
    }
}

pub async fn run_connected_loop(
    cs:       ConnectSuccess,
    config:   &Arc<Config>,
    buf:      &'static FailureBuffer,
    disk_buf: &Arc<DiskBuffer>,
    sampling: &Arc<AtomicU32>,
) -> DaemonState {
    let ConnectSuccess { out_tx, mut in_stream, .. } = cs;

    let tunnel_ctx = Arc::new(tunnel::TunnelContext::new(config.clone()));

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
                let status = MonitoringStatus {
                    disk_buffer_bytes:     disk_buf.used_bytes(),
                    disk_buffer_max_bytes: config.transmission.disk_buffer_max_bytes,
                    ..Default::default()
                };
                let msg = ClientMessage {
                    message: Some(client_message::Message::Heartbeat(Heartbeat {
                        seq:               hb_seq,
                        sent_at_unix_ms:   unix_ms_now(),
                        monitoring_status: Some(status),
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
                        if !handle_server_msg(msg, &out_tx, &mut in_flight, sampling, &tunnel_ctx).await {
                            return DaemonState::Connecting;
                        }
                    }
                }
            }
        }
    }
}

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

/// Returns `false` if the caller should reconnect immediately.
async fn handle_server_msg(
    msg:        ServerMessage,
    out_tx:     &mpsc::Sender<ClientMessage>,
    in_flight:  &mut HashSet<u64>,
    sampling:   &Arc<AtomicU32>,
    tunnel_ctx: &Arc<tunnel::TunnelContext>,
) -> bool {
    use server_message::Message as M;

    match msg.message {
        Some(M::HeartbeatAck(ack)) => {
            in_flight.remove(&ack.ack_seq);
        }
        Some(M::ServerHeartbeat(hb)) => {
            let _ = out_tx.send(ClientMessage {
                message: Some(client_message::Message::HeartbeatAck(HeartbeatAck {
                    ack_seq:          hb.seq,
                    acked_at_unix_ms: unix_ms_now(),
                })),
            }).await;
        }

        Some(M::ShutdownImminent(s)) => {
            let delay = Duration::from_millis(s.reconnect_after_ms as u64);
            info!("server shutting down; waiting {delay:?} before reconnecting");
            tokio::time::sleep(delay).await;
            return false;
        }

        Some(M::SetMonitoring(cmd)) => {
            info!(
                command_id        = %cmd.command_id,
                traffic_enabled   = cmd.traffic_enabled,
                telemetry_enabled = cmd.telemetry_enabled,
                "set_monitoring"
            );
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Success, "")).await;
        }
        Some(M::ThrottleMonitoring(t)) => {
            let rate = t.packet_sampling_rate.clamp(0.0, 1.0);
            sampling.store(rate.to_bits(), Ordering::Relaxed);
            info!(rate, "packet sampling rate updated");
        }

        Some(M::OpenSshTunnel(cmd)) => {
            let ctx  = tunnel_ctx.clone();
            let out  = out_tx.clone();
            let port = tunnel_ctx.config.agent.ssh_port;
            tokio::spawn(async move {
                match tunnel::transport::open_stream(&ctx, &cmd.tunnel_id).await {
                    Err(e) => {
                        let _ = out.send(cmd_result(
                            &cmd.command_id, CommandStatus::Failure,
                            &format!("SSH tunnel open failed: {e:#}"),
                        )).await;
                    }
                    Ok(stream) => {
                        let _ = out.send(cmd_result(
                            &cmd.command_id, CommandStatus::Success, "",
                        )).await;
                        if let Err(e) = tunnel::ssh::run_ssh_tunnel(stream, port).await {
                            tracing::debug!(command_id = %cmd.command_id, "SSH tunnel closed: {e}");
                        }
                    }
                }
            });
        }

        Some(M::OpenTtyTunnel(cmd)) => {
            let ctx   = tunnel_ctx.clone();
            let out   = out_tx.clone();
            let shell = tunnel_ctx.config.agent.tty_shell.clone();
            tokio::spawn(async move {
                match tunnel::transport::open_stream(&ctx, &cmd.tunnel_id).await {
                    Err(e) => {
                        let _ = out.send(cmd_result(
                            &cmd.command_id, CommandStatus::Failure,
                            &format!("TTY tunnel open failed: {e:#}"),
                        )).await;
                    }
                    Ok(stream) => {
                        let _ = out.send(cmd_result(
                            &cmd.command_id, CommandStatus::Success, "",
                        )).await;
                        if let Err(e) = tunnel::tty::run_tty_tunnel(stream, &shell).await {
                            tracing::debug!(command_id = %cmd.command_id, "TTY tunnel closed: {e}");
                        }
                    }
                }
            });
        }

        Some(M::OpenHttpTunnel(cmd)) => {
            let ctx         = tunnel_ctx.clone();
            let out         = out_tx.clone();
            let target_host = cmd.target_host.clone();
            let target_port = cmd.target_port as u16;
            tokio::spawn(async move {
                match tunnel::transport::open_stream(&ctx, &cmd.tunnel_id).await {
                    Err(e) => {
                        let _ = out.send(cmd_result(
                            &cmd.command_id, CommandStatus::Failure,
                            &format!("HTTP tunnel open failed: {e:#}"),
                        )).await;
                    }
                    Ok(stream) => {
                        let _ = out.send(cmd_result(
                            &cmd.command_id, CommandStatus::Success, "",
                        )).await;
                        if let Err(e) = tunnel::http::run_http_tunnel(stream, &target_host, target_port).await {
                            tracing::debug!(command_id = %cmd.command_id, "HTTP tunnel closed: {e}");
                        }
                    }
                }
            });
        }

        Some(M::OpenRemoteDesktopTunnel(cmd)) => {
            let _ = out_tx.send(cmd_result(
                &cmd.command_id, CommandStatus::Failure,
                "remote desktop: captis screen capture backend pending (Phase 8)",
            )).await;
        }

        // Firewall stubs (Phase 12)
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

        // Cert renewal stub (Phase 11)
        Some(M::RenewCertificateRequest(_)) => {
            warn!("cert renewal requested (Phase 11 stub) — ignoring");
        }

        Some(M::Welcome(_)) | Some(M::VersionRejected(_)) => {
            warn!("unexpected post-handshake message ignored");
        }

        None => {}
    }

    true
}
