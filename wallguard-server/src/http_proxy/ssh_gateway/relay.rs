use crate::app_context::AppContext;
use crate::datastore::RemoteAccessSession;

use super::ssh_session::SSHSession;
use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt as _;
use prost::bytes::Bytes;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// Starts bi-directional message relaying between a WebSocket session and an SSH session.
///
/// This function concurrently relays:
/// - Messages from the WebSocket client to the SSH server
/// - Messages from the SSH server to the WebSocket client
///
/// # Parameters
/// - `stream`: The WebSocket message stream.
/// - `ws_session`: The WebSocket session used to send messages back to the client.
/// - `ssh_session`: The SSH session used to read and write data.
/// - `ra_session`: The remote access session
/// - `context`: Application context
pub(crate) async fn relay(
    stream: MessageStream,
    ws_session: WSSession,
    ssh_session: SSHSession,
    remote_access_session: RemoteAccessSession,
    context: AppContext,
) {
    let stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let (tunnel_id, tunnel_terminate) = context
        .orchestractor
        .on_tunnel_established(&remote_access_session.id)
        .await;

    tokio::select! {
        _ = tunnel_terminate => {}
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

    context
        .orchestractor
        .on_tunnel_terminated(&remote_access_session.id, &tunnel_id)
        .await;
}

/// Relays incoming WebSocket messages to the SSH session's input stream.
///
/// Handles text, binary, and ping messages:
/// - Text/Binary messages are forwarded to the SSH session.
/// - Ping messages are responded to with Pong.
///
/// # Parameters
/// - `stream`: Aggregated WebSocket message stream.
/// - `ssh_session`: SSH session for writing received data.
/// - `ws_session`: WebSocket session used to respond to Ping messages.
async fn relay_messages_from_user_to_client(
    mut stream: AggregatedMessageStream,
    ssh_session: SSHSession,
    mut ws_session: WSSession,
) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(AggregatedMessage::Text(text)) => {
                if let Err(err) = ssh_session
                    .writer
                    .lock()
                    .await
                    .write_all(text.as_bytes())
                    .await
                {
                    log::error!("WS → SSH: Failed to write text: {err}");
                    return;
                } else {
                    log::debug!("WS → SSH: Sent text ({} bytes)", text.len());
                }
            }

            Ok(AggregatedMessage::Binary(bin)) => {
                if let Err(err) = ssh_session.writer.lock().await.write_all(&bin).await {
                    log::error!("WS → SSH: Failed to write binary: {err}");
                    return;
                } else {
                    log::debug!("WS → SSH: Sent binary ({} bytes)", bin.len());
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
                log::trace!("WS → SSH: Ignored unsupported message");
            }

            Err(err) => {
                log::error!("WS → SSH: Error reading WebSocket message: {err}");
                return;
            }
        }
    }

    log::info!("WS → SSH: WebSocket stream closed.");
}

/// Relays data read from the SSH session to the WebSocket session.
///
/// Reads raw bytes from the SSH session and sends them as binary messages
/// to the WebSocket client.
///
/// # Parameters
/// - `ws_session`: The WebSocket session to send binary data.
/// - `ssh_session`: The SSH session to read from.
async fn relay_messages_from_ssh_to_client(mut ws_session: WSSession, ssh_session: SSHSession) {
    loop {
        let mut buf = [0u8; 8196];
        match ssh_session.reader.lock().await.read(&mut buf).await {
            Ok(0) => {
                log::info!("SSH → WS: Reached EOF (client disconnected).");
                break;
            }
            Ok(n) => {
                let binmsg = Bytes::copy_from_slice(&buf[..n]);
                if let Err(err) = ws_session.binary(binmsg).await {
                    log::error!("SSH → WS: Failed to send binary message ({n} bytes): {err}");
                    break;
                } else {
                    log::debug!("SSH → WS: Sent binary ({n} bytes)");
                }
            }
            Err(err) => {
                log::error!("SSH → WS: Failed to read from SSH session: {err}");
                break;
            }
        }
    }

    log::info!("SSH → WS: SSH reader loop exited.");
}
