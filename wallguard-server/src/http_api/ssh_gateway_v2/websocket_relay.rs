use std::sync::Arc;

use crate::http_api::ssh_gateway_v2::session::Session as SshSession;
use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt as _;
use prost::bytes::Bytes;
use tokio::sync::Mutex;

pub async fn websocket_relay(
    stream: MessageStream,
    ws_session: WSSession,
    ssh_session: Arc<Mutex<SshSession>>,
) {
    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    tokio::select! {
        _ = relay_messages_from_user_to_client(
            stream,
            ssh_session.clone(),
            ws_session.clone()
        ) => {
            log::info!("WebSocket → SSH relay ended.");
        }
        _ = relay_messages_from_ssh_to_client(ws_session, ssh_session) => {
            log::info!("SSH → WebSocket relay ended.");
        }
    }
}

async fn relay_messages_from_user_to_client(
    mut stream: AggregatedMessageStream,
    ssh_session: Arc<Mutex<SshSession>>,
    mut ws_session: WSSession,
) {
    let sender = ssh_session.lock().await.get_data_send_channel();

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
    ssh_session: Arc<Mutex<SshSession>>,
) {
    let lock = ssh_session.lock().await;

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
