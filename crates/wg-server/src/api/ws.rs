use std::time::Duration;

use axum::{
    extract::{Extension, Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use axum::extract::ws::{Message, WebSocket};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::tunnels::new_command_id,
    middleware::auth::RequestContext,
    proto::control::{server_message, OpenSshTunnel, OpenTtyTunnel, ServerMessage},
    tunnel::TunnelStream,
    AppState,
};

// ---------------------------------------------------------------------------
// GET /api/v1/devices/:id/tunnels/ssh/:session_id  (WebSocket upgrade)
// ---------------------------------------------------------------------------

pub async fn ssh(
    ws:                               WebSocketUpgrade,
    Path((device_id, session_id)):    Path<(Uuid, Uuid)>,
    State(state):                     State<AppState>,
    Extension(_ctx):                  Extension<RequestContext>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ssh(socket, device_id, session_id, state))
}

async fn handle_ssh(socket: WebSocket, device_id: Uuid, session_id: Uuid, state: AppState) {
    // Register tunnel slot before sending the command.
    let stream_rx = state.tunnel_registry.register(&session_id.to_string()).await;

    // Send OpenSshTunnel command to the agent.
    let msg = ServerMessage {
        message: Some(server_message::Message::OpenSshTunnel(OpenSshTunnel {
            command_id: new_command_id(),
            tunnel_id:  session_id.to_string(),
            public_key: String::new(),
            username:   String::new(),
        })),
    };

    if !state.registry.send(&device_id, msg).await {
        tracing::warn!(%device_id, %session_id, "ssh: device not connected, closing ws");
        return;
    }

    // Wait up to 30s for the agent to open the reverse tunnel stream.
    let stream = match tokio::time::timeout(Duration::from_secs(30), stream_rx).await {
        Ok(Ok(s))  => s,
        Ok(Err(_)) => {
            tracing::warn!(%session_id, "ssh: tunnel registry channel dropped");
            return;
        }
        Err(_) => {
            tracing::warn!(%session_id, "ssh: timeout waiting for agent tunnel stream");
            return;
        }
    };

    relay_ws(socket, stream, state.pool, session_id).await;
}

// ---------------------------------------------------------------------------
// GET /api/v1/devices/:id/tunnels/tty/:session_id  (WebSocket upgrade)
// ---------------------------------------------------------------------------

pub async fn tty(
    ws:                               WebSocketUpgrade,
    Path((device_id, session_id)):    Path<(Uuid, Uuid)>,
    State(state):                     State<AppState>,
    Extension(_ctx):                  Extension<RequestContext>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_tty(socket, device_id, session_id, state))
}

async fn handle_tty(socket: WebSocket, device_id: Uuid, session_id: Uuid, state: AppState) {
    // Register tunnel slot before sending the command.
    let stream_rx = state.tunnel_registry.register(&session_id.to_string()).await;

    // Send OpenTtyTunnel command to the agent.
    let msg = ServerMessage {
        message: Some(server_message::Message::OpenTtyTunnel(OpenTtyTunnel {
            command_id: new_command_id(),
            tunnel_id:  session_id.to_string(),
        })),
    };

    if !state.registry.send(&device_id, msg).await {
        tracing::warn!(%device_id, %session_id, "tty: device not connected, closing ws");
        return;
    }

    // Wait up to 30s for the agent to open the reverse tunnel stream.
    let stream = match tokio::time::timeout(Duration::from_secs(30), stream_rx).await {
        Ok(Ok(s))  => s,
        Ok(Err(_)) => {
            tracing::warn!(%session_id, "tty: tunnel registry channel dropped");
            return;
        }
        Err(_) => {
            tracing::warn!(%session_id, "tty: timeout waiting for agent tunnel stream");
            return;
        }
    };

    relay_ws(socket, stream, state.pool, session_id).await;
}

// ---------------------------------------------------------------------------
// Bidirectional relay between a WebSocket connection and a TunnelStream
// ---------------------------------------------------------------------------

async fn relay_ws(mut socket: WebSocket, mut stream: TunnelStream, pool: PgPool, session_id: Uuid) {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    metrics::gauge!("wg_active_tunnels_total").increment(1.0);

    let mut buf = vec![0u8; 8192];
    loop {
        tokio::select! {
            result = stream.read.read(&mut buf) => {
                match result {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if socket.send(Message::Binary(buf[..n].to_vec())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if stream.write.write_all(&data).await.is_err() { break; }
                    }
                    Some(Ok(Message::Text(text))) => {
                        if stream.write.write_all(text.as_bytes()).await.is_err() { break; }
                    }
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                    _ => break,
                }
            }
        }
    }

    metrics::gauge!("wg_active_tunnels_total").decrement(1.0);

    // Close tunnel session in DB.
    sqlx::query("UPDATE tunnel_sessions SET status='closed', ended_at=NOW() WHERE id=$1")
        .bind(session_id)
        .execute(&pool)
        .await
        .ok();
}
