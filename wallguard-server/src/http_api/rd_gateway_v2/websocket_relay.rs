use std::sync::Arc;

use crate::{app_context::AppContext, datastore::TunnelStatus, tunneling::rd::RdTunnel};
use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt as _;
use prost::bytes::Bytes;
use tokio::sync::Mutex;

pub async fn websocket_relay(
    stream: MessageStream,
    ws_session: WSSession,
    rd_tunnel: Arc<Mutex<RdTunnel>>,
    context: Arc<AppContext>,
) {
    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    tokio::select! {
        _ = relay_user_to_rd(stream, rd_tunnel.clone(), ws_session.clone()) => {
            log::info!("WebSocket → RD relay ended.");
        }
        _ = relay_rd_to_user(ws_session, rd_tunnel.clone()) => {
            log::info!("RD → WebSocket relay ended.");
        }
    }

    let tunnel_lock = rd_tunnel.lock().await;

    if !tunnel_lock.has_active_viewers() {
        let tunnel_id = tunnel_lock.data.tunnel_data.id.clone();
        drop(tunnel_lock);

        if let Ok(token) = context.sysdev_token_provider.get().await {
            let _ = context
                .datastore
                .update_tunnel_status(&token.jwt, &tunnel_id, TunnelStatus::Idle, false)
                .await;
        }
    }
}

async fn relay_user_to_rd(
    mut stream: AggregatedMessageStream,
    rd_tunnel: Arc<Mutex<RdTunnel>>,
    mut ws_session: WSSession,
) {
    let sender = rd_tunnel.lock().await.get_data_send_channel();

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(AggregatedMessage::Text(text)) => {
                if sender.send(text.as_bytes().to_vec()).await.is_err() {
                    return;
                }
            }
            Ok(AggregatedMessage::Binary(bin)) => {
                if sender.send(bin.to_vec()).await.is_err() {
                    return;
                }
            }
            Ok(AggregatedMessage::Ping(msg)) => {
                if ws_session.pong(&msg).await.is_err() {
                    return;
                }
            }
            Ok(_) => continue,
            Err(_) => return,
        }
    }
}

async fn relay_rd_to_user(mut ws_session: WSSession, rd_tunnel: Arc<Mutex<RdTunnel>>) {
    let mut reader = rd_tunnel.lock().await.get_data_recv_channel();

    while let Ok(message) = reader.recv().await {
        let message = Bytes::copy_from_slice(&message);
        if ws_session.binary(message).await.is_err() {
            break;
        }
    }
}
