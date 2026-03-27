use std::sync::Arc;

use crate::{app_context::AppContext, datastore::TunnelStatus, tunneling::tty::TtyTunnel};
use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt as _;
use prost::bytes::Bytes;
use tokio::sync::Mutex;

pub async fn websocket_relay(
    stream: MessageStream,
    ws_session: WSSession,
    tty_tunnel: Arc<Mutex<TtyTunnel>>,
    context: Arc<AppContext>,
) {
    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    tokio::select! {
        _ = relay_messages_from_user_to_client(
            stream,
            tty_tunnel.clone(),
            ws_session.clone()
        ) => {
            log::info!("WebSocket → TTY relay ended.");
        }
        _ = relay_messages_from_ssh_to_client(ws_session, tty_tunnel.clone()) => {
            log::info!("TTY → WebSocket relay ended.");
        }
    }

    let tunnel_lock = tty_tunnel.lock().await;

    if !tunnel_lock.has_active_terminals() {
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

async fn relay_messages_from_user_to_client(
    mut stream: AggregatedMessageStream,
    tty_tunnel: Arc<Mutex<TtyTunnel>>,
    mut ws_session: WSSession,
) {
    let sender = tty_tunnel.lock().await.get_data_send_channel();

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

async fn relay_messages_from_ssh_to_client(
    mut ws_session: WSSession,
    tty_tunnel: Arc<Mutex<TtyTunnel>>,
) {
    let lock = tty_tunnel.lock().await;

    let memory = lock.get_memory_snaphot().await;
    {
        let message = Bytes::copy_from_slice(&memory);
        if ws_session.binary(message).await.is_err() {
            return;
        }
    }

    let mut reader = lock.get_data_recv_channel();

    drop(lock);

    while let Ok(message) = reader.recv().await {
        let message = Bytes::copy_from_slice(&message);
        if ws_session.binary(message).await.is_err() {
            break;
        }
    }
}
