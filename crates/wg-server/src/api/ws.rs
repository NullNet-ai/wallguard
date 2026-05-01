use std::time::Duration;

use axum::{
    extract::{Extension, Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use axum::extract::ws::{Message, WebSocket};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::tunnels::{new_command_id, RdpSessionParams},
    middleware::auth::RequestContext,
    proto::control::{server_message, OpenRemoteDesktopTunnel, OpenSshTunnel, OpenTtyTunnel, ServerMessage},
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
// GET /api/v1/devices/:id/tunnels/rdp/:session_id  (WebSocket upgrade)
// ---------------------------------------------------------------------------

pub async fn rdp(
    ws:                               WebSocketUpgrade,
    Path((device_id, session_id)):    Path<(Uuid, Uuid)>,
    State(state):                     State<AppState>,
    Extension(_ctx):                  Extension<RequestContext>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_rdp(socket, device_id, session_id, state))
}

async fn handle_rdp(socket: WebSocket, device_id: Uuid, session_id: Uuid, state: AppState) {
    let params: RdpSessionParams = match state.pending_rdp.lock().await.remove(&session_id) {
        Some(p) => p,
        None => {
            tracing::warn!(%session_id, "rdp: no pending session params found");
            return;
        }
    };

    let stream_rx = state.tunnel_registry.register(&session_id.to_string()).await;

    let msg = ServerMessage {
        message: Some(server_message::Message::OpenRemoteDesktopTunnel(OpenRemoteDesktopTunnel {
            command_id:  new_command_id(),
            tunnel_id:   session_id.to_string(),
            width:       params.width,
            height:      params.height,
            target_fps:  params.target_fps,
            target_kbps: params.target_kbps,
        })),
    };

    if !state.registry.send(&device_id, msg).await {
        tracing::warn!(%device_id, %session_id, "rdp: device not connected, closing ws");
        return;
    }

    let stream = match tokio::time::timeout(Duration::from_secs(30), stream_rx).await {
        Ok(Ok(s))  => s,
        Ok(Err(_)) => {
            tracing::warn!(%session_id, "rdp: tunnel registry channel dropped");
            return;
        }
        Err(_) => {
            tracing::warn!(%session_id, "rdp: timeout waiting for agent tunnel stream");
            return;
        }
    };

    relay_rdp_ws(socket, stream, state.pool, session_id).await;
}

/// Relay between the browser WebSocket and the agent QUIC stream.
///
/// Agent → browser: strips the 4-byte LE length prefix and sends raw NAL bytes as Binary.
/// Browser → agent: wraps JSON text (or binary) input events with a 4-byte LE length prefix.
async fn relay_rdp_ws(mut socket: WebSocket, stream: TunnelStream, pool: PgPool, session_id: Uuid) {
    use tokio::io::AsyncWriteExt as _;

    metrics::gauge!("wg_active_tunnels_total").increment(1.0);

    let stream_read   = stream.read;
    let mut stream_write = stream.write;

    // Dedicated task: reads framed NAL chunks from the QUIC stream and forwards
    // them as raw vecs (length prefix stripped) via channel.
    let (nal_tx, mut nal_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(4);
    let reader_handle = tokio::spawn(async move {
        use tokio::io::AsyncReadExt as _;
        let mut r = stream_read;
        let mut len_buf = [0u8; 4];
        loop {
            if r.read_exact(&mut len_buf).await.is_err() { break; }
            let len = u32::from_le_bytes(len_buf) as usize;
            if len > 4 * 1024 * 1024 { break; } // 4 MiB sanity limit per NAL
            let mut data = vec![0u8; len];
            if r.read_exact(&mut data).await.is_err() { break; }
            if nal_tx.send(data).await.is_err() { break; }
        }
    });

    'relay: loop {
        tokio::select! {
            nal = nal_rx.recv() => {
                match nal {
                    None => break,
                    Some(data) => {
                        if socket.send(Message::Binary(data)).await.is_err() {
                            break 'relay;
                        }
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let bytes = text.as_bytes();
                        let len   = (bytes.len() as u32).to_le_bytes();
                        if stream_write.write_all(&len).await.is_err()   { break 'relay; }
                        if stream_write.write_all(bytes).await.is_err()  { break 'relay; }
                        if stream_write.flush().await.is_err()           { break 'relay; }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        let len = (data.len() as u32).to_le_bytes();
                        if stream_write.write_all(&len).await.is_err()   { break 'relay; }
                        if stream_write.write_all(&data).await.is_err()  { break 'relay; }
                        if stream_write.flush().await.is_err()           { break 'relay; }
                    }
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                    _ => break,
                }
            }
        }
    }

    reader_handle.abort();
    metrics::gauge!("wg_active_tunnels_total").decrement(1.0);

    sqlx::query("UPDATE tunnel_sessions SET status='closed', ended_at=NOW() WHERE id=$1")
        .bind(session_id)
        .execute(&pool)
        .await
        .ok();
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
                        let text = String::from_utf8_lossy(&buf[..n]).into_owned();
                        if socket.send(Message::Text(text)).await.is_err() {
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
