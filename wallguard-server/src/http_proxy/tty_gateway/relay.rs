use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt as _;
use prost::bytes::Bytes;
use wallguard_common::protobuf::wallguard_tunnel::client_frame::Message as ClientMessage;

use crate::reverse_tunnel::TunnelInstance;

pub(crate) async fn relay(
    msg_stream: MessageStream,
    ws_session: WSSession,
    tty_tunnel: TunnelInstance,
) {
    let stream = msg_stream
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
        _ = relay_messages_from_ssh_to_client(ws_session, tty_tunnel) => {
            log::info!("TTY → WebSocket relay ended.");
        }
    }
}

async fn relay_messages_from_user_to_client(
    mut stream: AggregatedMessageStream,
    tty_tunnel: TunnelInstance,
    mut ws_session: WSSession,
) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(AggregatedMessage::Text(text)) => {
                let data_frame = TunnelInstance::make_data_frame(text.as_bytes());
                if let Err(err) = tty_tunnel.write(data_frame).await {
                    log::error!("WS → TTY: Failed to write text: {}", err.to_str());
                    return;
                } else {
                    log::debug!("WS → TTY: Sent text ({} bytes)", text.len());
                }
            }

            Ok(AggregatedMessage::Binary(bin)) => {
                let data_frame = TunnelInstance::make_data_frame(&bin);
                if let Err(err) = tty_tunnel.write(data_frame).await {
                    log::error!("WS → TTY: Failed to write binary: {}", err.to_str());
                    return;
                } else {
                    log::debug!("WS → TTY: Sent binary ({} bytes)", bin.len());
                }
            }

            Ok(AggregatedMessage::Ping(msg)) => {
                if let Err(err) = ws_session.pong(&msg).await {
                    log::error!("WS → WS: Failed to respond to ping: {err}");
                    return;
                } else {
                    log::debug!("WS → WS: Responded to ping");
                }
            }

            Ok(_) => {
                log::trace!("WS → TTY: Ignored unsupported message");
            }

            Err(err) => {
                log::error!("WS → TTY: Error reading WebSocket message: {err}");
                return;
            }
        }
    }

    log::info!("WS → SSH: WebSocket stream closed.");
}

async fn relay_messages_from_ssh_to_client(mut ws_session: WSSession, tty_tunnel: TunnelInstance) {
    loop {
        let Ok(message) = tty_tunnel.read().await else {
            log::error!("TTY → WS: Failed to read from TTY session");
            break;
        };

        let Some(message) = message.message else {
            log::info!("TTY → WS: Reached EOF (client disconnected).");
            break;
        };

        let ClientMessage::Data(data) = message else {
            log::error!("TTY → WS: Unexpected message.");
            break;
        };

        let message_for_ws = Bytes::copy_from_slice(&data.data);

        if let Err(err) = ws_session.binary(message_for_ws).await {
            log::error!("TTY → WS: Failed to send binary message: {err}");
            break;
        } else {
            log::debug!("TTY → WS: Sent binary ({} bytes)", data.data.len());
        }
    }

    log::info!("TTY → WS: TTY reader loop exited.");
}
