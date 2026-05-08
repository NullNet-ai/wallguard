use std::pin::Pin;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use serde_json;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;
use tonic::{Request, Response, Status, Streaming};
use tracing::Instrument;
use uuid::Uuid;
use wg_shared::capabilities::{MIN_AGENT_PROTOCOL_VERSION, PROTOCOL_VERSION};

use crate::connection_registry::{DeviceConnection, DeviceId};
use crate::events::{SseEvent, SseEventKind};
use crate::grpc::extract_device_id;
use crate::heartbeat::{self, HeartbeatState};
use crate::proto::control::{
    client_message, control_server::Control, server_message,
    ClientMessage, CommandResult, CommandStatus, DeviceSettings, FailureCategory, FailureSeverity,
    Heartbeat, HeartbeatAck, ServerMessage, VersionRejected, Welcome,
};
use crate::AppState;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// gRPC service
// ---------------------------------------------------------------------------

pub struct ControlService {
    pub state: AppState,
}

pub use crate::proto::control::control_server::ControlServer;

type ChannelStream = Pin<Box<dyn tonic::codegen::tokio_stream::Stream<
    Item = Result<ServerMessage, Status>,
> + Send + 'static>>;

#[tonic::async_trait]
impl Control for ControlService {
    type ChannelStream = ChannelStream;

    async fn channel(
        &self,
        request: Request<Streaming<ClientMessage>>,
    ) -> Result<Response<ChannelStream>, Status> {
        let device_id = extract_device_id(&request)
            .ok_or_else(|| Status::unauthenticated("no valid device certificate"))?;

        let (out_tx, out_rx) = mpsc::channel::<ServerMessage>(64);
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        // Register the connection — replaces any stale connection for this device.
        let conn = DeviceConnection {
            org_id:       Uuid::nil(),   // updated after DB lookup in the task
            out_tx:       out_tx.clone(),
            connected_at: std::time::Instant::now(),
            shutdown_tx:  shutdown_tx.clone(),
        };
        self.state.registry.insert(device_id, conn).await;

        let state   = self.state.clone();
        let in_st   = request.into_inner();
        let mut shutdown_rx = shutdown_tx.subscribe();

        let span = tracing::info_span!("connection", device_id = %device_id);
        tokio::spawn(async move {
            run_connection(device_id, in_st, out_tx, &mut shutdown_rx, &state).await;
            state.registry.remove(&device_id).await;
            tracing::info!(%device_id, "connection closed");
        }.instrument(span));

        let out_stream = ReceiverStream::new(out_rx).map(Ok);
        Ok(Response::new(Box::pin(out_stream)))
    }
}

// ---------------------------------------------------------------------------
// Per-connection task
// ---------------------------------------------------------------------------

async fn run_connection(
    device_id:   DeviceId,
    mut in_st:   Streaming<ClientMessage>,
    out_tx:      mpsc::Sender<ServerMessage>,
    shutdown_rx: &mut broadcast::Receiver<()>,
    state:       &AppState,
) {
    // ── 1. Handshake ──────────────────────────────────────────────────────
    let (org_id, device_name) = match handshake(device_id, &mut in_st, &out_tx, state).await {
        Ok(pair) => pair,
        Err(e)   => {
            tracing::warn!(%device_id, "handshake failed: {e}");
            return;
        }
    };

    tracing::info!(%device_id, %org_id, "connected");
    metrics::gauge!("wg_connected_agents_total").increment(1.0);
    let _ = state.sse_tx.send(SseEvent {
        org_id,
        kind: SseEventKind::DeviceConnected { device_id, device_name: device_name.clone() },
    });

    // ── 2. Message loop ────────────────────────────────────────────────────
    let mut hb_timer = tokio::time::interval(HEARTBEAT_INTERVAL);
    let mut hb       = HeartbeatState::new();

    loop {
        tokio::select! {
            biased;

            _ = shutdown_rx.recv() => {
                tracing::debug!(%device_id, "connection shutdown signal");
                break;
            }

            _ = hb_timer.tick() => {
                if hb.should_disconnect() {
                    tracing::warn!(%device_id, "3 heartbeat acks missed — closing");
                    break;
                }
                let seq = hb.next_seq();
                let _ = out_tx.send(ServerMessage {
                    message: Some(server_message::Message::ServerHeartbeat(Heartbeat {
                        seq,
                        sent_at_unix_ms:   unix_ms_now(),
                        monitoring_status: None,
                    })),
                }).await;
            }

            msg = in_st.message() => {
                match msg {
                    Err(e)    => { tracing::warn!(%device_id, "stream error: {e}"); break; }
                    Ok(None)  => { tracing::info!(%device_id, "agent disconnected"); break; }
                    Ok(Some(msg)) => {
                        handle_client_msg(
                            msg, device_id, org_id, &device_name, &out_tx, &mut hb, state,
                        ).await;
                    }
                }
            }
        }
    }

    metrics::gauge!("wg_connected_agents_total").decrement(1.0);
    let _ = state.sse_tx.send(SseEvent {
        org_id,
        kind: SseEventKind::DeviceDisconnected { device_id, device_name },
    });
}

// ---------------------------------------------------------------------------
// Handshake
// ---------------------------------------------------------------------------

async fn handshake(
    device_id: DeviceId,
    in_st:     &mut Streaming<ClientMessage>,
    out_tx:    &mpsc::Sender<ServerMessage>,
    state:     &AppState,
) -> anyhow::Result<(Uuid, String)> {
    let msg = in_st.message().await?
        .ok_or_else(|| anyhow::anyhow!("stream closed before Hello"))?;

    let hello = match msg.message {
        Some(client_message::Message::Hello(h)) => h,
        _ => anyhow::bail!("expected Hello, got something else"),
    };

    // Version check.
    if hello.protocol_version < MIN_AGENT_PROTOCOL_VERSION {
        let _ = out_tx.send(ServerMessage {
            message: Some(server_message::Message::VersionRejected(VersionRejected {
                min_required_version: MIN_AGENT_PROTOCOL_VERSION,
                message: format!(
                    "upgrade agent to protocol version {MIN_AGENT_PROTOCOL_VERSION}"
                ),
            })),
        }).await;
        anyhow::bail!("agent protocol version {} too old", hello.protocol_version);
    }

    // Look up the device and org in the DB.
    let (org_id, device_name): (Uuid, String) = sqlx::query_as(
        "SELECT org_id, display_name FROM devices WHERE id = $1",
    )
    .bind(device_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("device {device_id} not in DB"))?;

    // Feature negotiation — for now accept everything the agent advertises.
    let negotiated = hello.supported_features.clone();

    // Update device record.
    let feature_strings: Vec<String> = hello
        .supported_features
        .iter()
        .filter_map(|&f| proto_feature_to_str(f))
        .map(String::from)
        .collect();
    let system_info_json = hello.system_info.as_ref().map(proto_system_info_to_json);

    sqlx::query(
        "UPDATE devices SET last_seen_at = NOW(), features = $1, system_info = $2 WHERE id = $3",
    )
    .bind(&feature_strings)
    .bind(&system_info_json)
    .bind(device_id)
    .execute(&state.pool)
    .await
    .ok();

    // Send Welcome.
    let _ = out_tx.send(ServerMessage {
        message: Some(server_message::Message::Welcome(Welcome {
            protocol_version:    PROTOCOL_VERSION,
            negotiated_features: negotiated,
            initial_settings:    Some(DeviceSettings {
                traffic_monitoring_enabled:   true,
                telemetry_monitoring_enabled: true,
                config_monitoring_enabled:    false,
                packet_sampling_rate:         1.0,
            }),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        })),
    }).await;

    Ok((org_id, device_name))
}

// ---------------------------------------------------------------------------
// Incoming message dispatch
// ---------------------------------------------------------------------------

async fn handle_client_msg(
    msg:         ClientMessage,
    device_id:   DeviceId,
    org_id:      Uuid,
    device_name: &str,
    out_tx:      &mpsc::Sender<ServerMessage>,
    hb:          &mut HeartbeatState,
    state:       &AppState,
) {
    use client_message::Message as M;

    let span = tracing::debug_span!("command", device_id = %device_id);
    let _g = span.enter();

    match msg.message {
        // ── Heartbeat (agent → server) ────────────────────────────────────
        Some(M::Heartbeat(agent_hb)) => {
            // Ack the agent's heartbeat.
            let _ = out_tx.send(ServerMessage {
                message: Some(server_message::Message::HeartbeatAck(HeartbeatAck {
                    ack_seq:          agent_hb.seq,
                    acked_at_unix_ms: unix_ms_now(),
                })),
            }).await;

            // Update monitoring status in the registry + throttled DB write.
            if let Some(status) = agent_hb.monitoring_status {
                if hb.should_write_db() {
                    heartbeat::record_monitoring_status(&state.pool, device_id, &status).await;
                }
            }
        }

        // ── Heartbeat ack (agent acks our heartbeat) ──────────────────────
        Some(M::HeartbeatAck(ack)) => {
            hb.on_ack(ack.ack_seq);
        }

        // ── Command results ───────────────────────────────────────────────
        Some(M::CommandResult(result)) => {
            resolve_command_result(&result, state).await;
        }

        // ── Failures (buffered replay or live) ────────────────────────────
        Some(M::AgentFailure(failure)) => {
            let severity_str = match FailureSeverity::try_from(failure.severity)
                .unwrap_or(FailureSeverity::Warning)
            {
                FailureSeverity::Warning => "warning",
                FailureSeverity::Error   => "error",
                FailureSeverity::Fatal   => "fatal",
            };
            let category_str = match FailureCategory::try_from(failure.category)
                .unwrap_or(FailureCategory::Monitoring)
            {
                FailureCategory::Monitoring   => "monitoring",
                FailureCategory::Tunnel       => "tunnel",
                FailureCategory::DiskBuffer   => "disk_buffer",
                FailureCategory::Fireparse    => "fireparse",
                FailureCategory::AgentCrash   => "agent_crash",
                FailureCategory::Connectivity => "connectivity",
                FailureCategory::System       => "system",
            };
            tracing::info!(
                %device_id,
                failure_id = %failure.failure_id,
                severity   = severity_str,
                is_replay  = failure.is_replay,
                "{}",
                failure.message,
            );
            let failure_id = Uuid::parse_str(&failure.failure_id)
                .unwrap_or_else(|_| Uuid::new_v4());
            let occurred_at = time::OffsetDateTime::from_unix_timestamp_nanos(
                failure.occurred_at as i128 * 1_000_000,
            )
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
            let context_val: Option<serde_json::Value> =
                serde_json::from_str(&failure.context).ok();
            sqlx::query(
                r#"INSERT INTO device_failures
                    (failure_id, device_id, severity, category, message, context, occurred_at, is_replay)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                   ON CONFLICT (failure_id) DO NOTHING"#,
            )
            .bind(failure_id)
            .bind(device_id)
            .bind(severity_str)
            .bind(category_str)
            .bind(&failure.message)
            .bind(context_val)
            .bind(occurred_at)
            .bind(failure.is_replay)
            .execute(&state.pool)
            .await
            .ok();
            metrics::counter!("wg_agent_failures_total", "severity" => severity_str.to_string())
                .increment(1);
            let _ = state.sse_tx.send(SseEvent {
                org_id,
                kind: SseEventKind::NewFailure {
                    device_id,
                    device_name: device_name.to_string(),
                    failure_id,
                    severity: severity_str.to_string(),
                    message: failure.message.clone(),
                },
            });
        }

        // ── Certificate renewal (Phase 11) ────────────────────────────────
        Some(M::RenewCertificateResponse(_)) => {
            tracing::info!(%device_id, "cert renewal response (Phase 11 stub)");
        }

        // ── Config snapshot (Phase 12) ────────────────────────────────────
        Some(M::ConfigSnapshot(snap)) => {
            tracing::info!(%device_id, digest = %snap.digest, "config snapshot received (Phase 12 stub)");
        }

        // ── HTTP service advertisement ─────────────────────────────────────
        Some(M::HttpServicesUpdate(update)) => {
            state.registry.update_http_services(device_id, update.services).await;
            let _ = state.sse_tx.send(SseEvent {
                org_id,
                kind: SseEventKind::HttpServicesUpdated { device_id, device_name: device_name.to_string() },
            });
        }

        // Hello only valid at handshake time.
        Some(M::Hello(_)) => {
            tracing::warn!(%device_id, "duplicate Hello ignored");
        }

        None => {}
    }
}

// ---------------------------------------------------------------------------
// Command result resolution
// ---------------------------------------------------------------------------

async fn resolve_command_result(result: &CommandResult, state: &AppState) {
    use crate::command_tracker::CommandOutcome;

    let status_label = match CommandStatus::try_from(result.status).unwrap_or(CommandStatus::Failure) {
        CommandStatus::Success => "success",
        CommandStatus::Failure => "failure",
        CommandStatus::Timeout => "timeout",
    };

    let outcome = match CommandStatus::try_from(result.status).unwrap_or(CommandStatus::Failure) {
        CommandStatus::Success => CommandOutcome::Success {
            output:         result.output.clone(),
            applied_digest: result.applied_digest.clone(),
        },
        CommandStatus::Failure => CommandOutcome::Failure {
            error_message: result.error_message.clone(),
        },
        CommandStatus::Timeout => CommandOutcome::Timeout,
    };

    metrics::counter!("wg_commands_resolved_total", "status" => status_label).increment(1);
    state.tracker.resolve(&result.command_id, outcome).await;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn proto_system_info_to_json(si: &crate::proto::control::SystemInfo) -> serde_json::Value {
    use wg_shared::types::{DiskInfo, NetInterface, SystemInfo};
    let info = SystemInfo {
        hostname:        si.hostname.clone(),
        os_name:         si.os_name.clone(),
        os_version:      si.os_version.clone(),
        kernel_version:  si.kernel_version.clone(),
        arch:            si.arch.clone(),
        cpu_brand:       si.cpu_brand.clone(),
        cpu_cores:       si.cpu_cores,
        total_mem_bytes: si.total_mem_bytes,
        disks: si.disks.iter().map(|d| DiskInfo {
            name:        d.name.clone(),
            total_bytes: d.total_bytes,
        }).collect(),
        interfaces: si.interfaces.iter().map(|i| NetInterface {
            name: i.name.clone(),
            mac:  i.mac.clone(),
        }).collect(),
    };
    serde_json::to_value(info).unwrap_or(serde_json::Value::Null)
}

fn proto_feature_to_str(f: i32) -> Option<&'static str> {
    use crate::proto::control::Feature;
    match Feature::try_from(f).ok()? {
        Feature::NetworkMonitoring   => Some("network_monitoring"),
        Feature::TelemetryMonitoring => Some("telemetry_monitoring"),
        Feature::ConfigMonitoring    => Some("config_monitoring"),
        Feature::SshTunnel           => Some("ssh_tunnel"),
        Feature::TtyTunnel           => Some("tty_tunnel"),
        Feature::HttpTunnel          => Some("http_tunnel"),
        Feature::NamedCommands       => Some("named_commands"),
        Feature::RemoteDesktop       => Some("remote_desktop"),
    }
}

fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
