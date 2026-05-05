use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};
use wg_shared::types::Feature;

use crate::config::Config;
use crate::disk_buffer::DiskBuffer;
use crate::failure_buffer::FailureBuffer;
use crate::pipeline::control::PipelineControl;
use crate::proto::control::{
    client_message, control_client::ControlClient, server_message, ClientMessage, CommandStatus,
    Heartbeat, HeartbeatAck, HttpServicesUpdate, MonitoringStatus, RenewCertificateResponse,
    ServerMessage,
};
use crate::proto_conv::{
    cmd_result, failure_entry_to_proto, make_hello, proto_to_shared_feature, unix_ms_now,
};
use crate::state::DaemonState;
use crate::tunnel::{self, TunnelStream};

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
    let channel = crate::tls::build_grpc_channel(config, config.grpc_endpoint()).await?;

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
    ctrl:     &Arc<PipelineControl>,
    tls:      &Arc<rustls::ClientConfig>,
) -> DaemonState {
    let ConnectSuccess { out_tx, mut in_stream, .. } = cs;

    let tunnel_ctx = Arc::new(tunnel::TunnelContext::new(config.clone(), tls.clone()));

    replay_failures(&out_tx, buf).await;

    let hb_interval  = Duration::from_secs(config.agent.heartbeat_interval_s);
    let mut hb_timer = tokio::time::interval(hb_interval);
    let mut hb_seq   = 0u64;
    let mut in_flight: HashSet<u64> = HashSet::new();
    let mut pending_cert_key: Option<String> = None;

    let mut scan_timer = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = scan_timer.tick() => {
                let cfg = config.clone();
                let tx  = out_tx.clone();
                tokio::spawn(async move {
                    let services = crate::http_scanner::scan(&cfg).await;
                    let _ = tx.send(ClientMessage {
                        message: Some(client_message::Message::HttpServicesUpdate(
                            HttpServicesUpdate { services },
                        )),
                    }).await;
                });
            }

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
                        if !handle_server_msg(msg, &out_tx, &mut in_flight, ctrl, &tunnel_ctx, &mut pending_cert_key).await {
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

/// Spawn a tunnel task following the standard open→result→relay pattern.
///
/// All four tunnel types (SSH, TTY, HTTP, RDP) share this structure:
/// open a transport stream, send CommandResult::Success/Failure, then hand
/// the stream to the protocol-specific relay function.
fn spawn_tunnel<F, Fut>(
    tunnel_id:  String,
    command_id: String,
    err_prefix: &'static str,
    out_tx:     mpsc::Sender<ClientMessage>,
    ctx:        Arc<tunnel::TunnelContext>,
    run:        F,
)
where
    F:   FnOnce(TunnelStream) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        let _guard = crate::lifecycle::upgrade::InFlightGuard::new();
        match tunnel::transport::open_stream(&ctx, &tunnel_id).await {
            Err(e) => {
                let _ = out_tx.send(cmd_result(
                    &command_id,
                    CommandStatus::Failure,
                    &format!("{err_prefix}: {e:#}"),
                )).await;
            }
            Ok(stream) => {
                let _ = out_tx.send(cmd_result(&command_id, CommandStatus::Success, "")).await;
                if let Err(e) = run(stream).await {
                    tracing::debug!(command_id = %command_id, "tunnel closed: {e:#}");
                }
            }
        }
    });
}

/// Returns `false` if the caller should reconnect immediately.
async fn handle_server_msg(
    msg:              ServerMessage,
    out_tx:           &mpsc::Sender<ClientMessage>,
    in_flight:        &mut HashSet<u64>,
    ctrl:             &Arc<PipelineControl>,
    tunnel_ctx:       &Arc<tunnel::TunnelContext>,
    pending_cert_key: &mut Option<String>,
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
            ctrl.set_telemetry_enabled(cmd.telemetry_enabled);
            info!(
                command_id        = %cmd.command_id,
                traffic_enabled   = cmd.traffic_enabled,
                telemetry_enabled = cmd.telemetry_enabled,
                "set_monitoring"
            );
            let _ = out_tx.send(cmd_result(&cmd.command_id, CommandStatus::Success, "")).await;
        }
        Some(M::ThrottleMonitoring(t)) => {
            let rate = t.packet_sampling_rate;
            ctrl.set_sampling_rate(rate);
            info!(rate, "packet sampling rate updated");
        }

        Some(M::OpenSshTunnel(cmd)) => {
            let port     = tunnel_ctx.config.agent.ssh_port;
            let username = cmd.username.clone();
            spawn_tunnel(
                cmd.tunnel_id, cmd.command_id, "SSH tunnel open failed",
                out_tx.clone(), tunnel_ctx.clone(),
                move |stream| async move {
                    tunnel::ssh::run_ssh_tunnel(stream, port, &username).await
                },
            );
        }

        Some(M::OpenTtyTunnel(cmd)) => {
            let shell = tunnel_ctx.config.agent.tty_shell.clone();
            spawn_tunnel(
                cmd.tunnel_id, cmd.command_id, "TTY tunnel open failed",
                out_tx.clone(), tunnel_ctx.clone(),
                move |stream| async move {
                    tunnel::tty::run_tty_tunnel(stream, &shell).await
                },
            );
        }

        Some(M::OpenHttpTunnel(cmd)) => {
            let target_host = cmd.target_host.clone();
            let target_port = cmd.target_port as u16;
            spawn_tunnel(
                cmd.tunnel_id, cmd.command_id, "HTTP tunnel open failed",
                out_tx.clone(), tunnel_ctx.clone(),
                move |stream| async move {
                    tunnel::http::run_http_tunnel(stream, &target_host, target_port).await
                },
            );
        }

        Some(M::OpenRemoteDesktopTunnel(cmd)) => {
            let (w, h, fps, kbps) = (cmd.width, cmd.height, cmd.target_fps, cmd.target_kbps);
            spawn_tunnel(
                cmd.tunnel_id, cmd.command_id, "RDP tunnel open failed",
                out_tx.clone(), tunnel_ctx.clone(),
                move |stream| async move {
                    tunnel::remote_desktop::run_remote_desktop_tunnel(stream, w, h, fps, kbps).await
                },
            );
        }

        Some(M::CloseRemoteDesktopTunnel(cmd)) => {
            tracing::debug!(session_id = %cmd.session_id, "CloseRemoteDesktopTunnel received");
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

        Some(M::RenewCertificateRequest(_)) => {
            let device_id = tunnel_ctx.config.device.id.clone();
            match crate::lifecycle::cert_renewal::generate_csr(&device_id) {
                Ok((key_pem, csr_pem)) => {
                    info!("generated CSR for cert renewal");
                    *pending_cert_key = Some(key_pem);
                    let _ = out_tx.send(ClientMessage {
                        message: Some(client_message::Message::RenewCertificateResponse(
                            RenewCertificateResponse { csr_pem },
                        )),
                    }).await;
                }
                Err(e) => warn!("cert renewal: CSR generation failed: {e:#}"),
            }
        }

        Some(M::SetCertificate(cmd)) => {
            match pending_cert_key.take() {
                None => warn!("received SetCertificate without a pending CSR — ignoring"),
                Some(key_pem) => {
                    let cfg = &tunnel_ctx.config.tls;
                    match crate::lifecycle::cert_renewal::install_cert(
                        &cmd.cert_pem,
                        &cmd.ca_pem,
                        &key_pem,
                        &cfg.device_cert,
                        &cfg.ca_cert,
                        &cfg.device_key,
                    ) {
                        Ok(_) => {
                            info!("new certificate installed — reconnecting with renewed cert");
                            return false;
                        }
                        Err(e) => warn!("cert renewal: install failed: {e:#}"),
                    }
                }
            }
        }

        Some(M::Welcome(_)) | Some(M::VersionRejected(_)) => {
            warn!("unexpected post-handshake message ignored");
        }

        None => {}
    }

    true
}
