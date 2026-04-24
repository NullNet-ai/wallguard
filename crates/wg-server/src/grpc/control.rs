use std::pin::Pin;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;
use wg_shared::capabilities::{MIN_AGENT_PROTOCOL_VERSION, PROTOCOL_VERSION};

use crate::connection_registry::{DeviceConnection, DeviceId};
use crate::heartbeat::{self, HeartbeatState};
use crate::proto::control::{
    client_message, control_server::Control, server_message,
    ClientMessage, CommandResult, CommandStatus, DeviceSettings, Heartbeat, HeartbeatAck,
    ServerMessage, VersionRejected, Welcome,
};
use crate::AppState;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Peer-cert extraction
// ---------------------------------------------------------------------------

fn extract_device_id(request: &Request<Streaming<ClientMessage>>) -> Option<DeviceId> {
    let certs = request.peer_certs()?;
    let der   = certs.first()?.as_ref();
    extract_device_id_from_der(der)
}

fn extract_device_id_from_der(der: &[u8]) -> Option<DeviceId> {
    use x509_parser::prelude::*;
    let (_, cert) = X509Certificate::from_der(der).ok()?;
    let cn = cert.subject().iter_common_name().next()?.as_str().ok()?;
    Uuid::parse_str(cn.strip_prefix("device:")?).ok()
}

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

        tokio::spawn(async move {
            run_connection(device_id, in_st, out_tx, &mut shutdown_rx, &state).await;
            state.registry.remove(&device_id).await;
            tracing::info!(%device_id, "connection closed");
        });

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
    let org_id = match handshake(device_id, &mut in_st, &out_tx, state).await {
        Ok(id)   => id,
        Err(e)   => {
            tracing::warn!(%device_id, "handshake failed: {e}");
            return;
        }
    };

    tracing::info!(%device_id, %org_id, "connected");

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
                            msg, device_id, &out_tx, &mut hb, state,
                        ).await;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Handshake
// ---------------------------------------------------------------------------

async fn handshake(
    device_id: DeviceId,
    in_st:     &mut Streaming<ClientMessage>,
    out_tx:    &mpsc::Sender<ServerMessage>,
    state:     &AppState,
) -> anyhow::Result<Uuid> {
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
    let org_id: Uuid = sqlx::query_scalar(
        "SELECT org_id FROM devices WHERE id = $1",
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
        .map(|&f| format!("{f}"))
        .collect();
    sqlx::query(
        "UPDATE devices SET last_seen_at = NOW(), features = $1 WHERE id = $2",
    )
    .bind(&feature_strings)
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

    Ok(org_id)
}

// ---------------------------------------------------------------------------
// Incoming message dispatch
// ---------------------------------------------------------------------------

async fn handle_client_msg(
    msg:       ClientMessage,
    device_id: DeviceId,
    out_tx:    &mpsc::Sender<ServerMessage>,
    hb:        &mut HeartbeatState,
    state:     &AppState,
) {
    use client_message::Message as M;

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
            tracing::info!(
                %device_id,
                failure_id = %failure.failure_id,
                severity   = failure.severity,
                is_replay  = failure.is_replay,
                "{}",
                failure.message,
            );
            // Phase 9: persist to agent_failures table.
        }

        // ── Certificate renewal (Phase 11) ────────────────────────────────
        Some(M::RenewCertificateResponse(_)) => {
            tracing::info!(%device_id, "cert renewal response (Phase 11 stub)");
        }

        // ── Config snapshot (Phase 12) ────────────────────────────────────
        Some(M::ConfigSnapshot(snap)) => {
            tracing::info!(%device_id, digest = %snap.digest, "config snapshot received (Phase 12 stub)");
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

    state.tracker.resolve(&result.command_id, outcome).await;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
