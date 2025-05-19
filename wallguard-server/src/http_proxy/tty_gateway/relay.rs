use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt as _;
use prost::bytes::Bytes;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;

pub(crate) async fn relay(msg_stream: MessageStream, ws_session: WSSession, tty_stream: TcpStream) {
    let stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let (tty_reader, tty_writer) = tokio::io::split(tty_stream);

    tokio::select! {
        _ = relay_messages_from_user_to_client(
            stream,
            tty_writer,
            ws_session.clone()
        ) => {
            log::info!("WebSocket → TTY relay ended.");
        }
        _ = relay_messages_from_ssh_to_client(ws_session, tty_reader) => {
            log::info!("TTY → WebSocket relay ended.");
        }
    }
}

async fn relay_messages_from_user_to_client(
    mut stream: AggregatedMessageStream,
    mut tty_writer: WriteHalf<TcpStream>,
    mut ws_session: WSSession,
) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(AggregatedMessage::Text(text)) => {
                if let Err(err) = tty_writer.write_all(text.as_bytes()).await {
                    log::error!("WS → TTY: Failed to write text: {}", err);
                    return;
                } else {
                    log::debug!("WS → TTY: Sent text ({} bytes)", text.len());
                }
            }

            Ok(AggregatedMessage::Binary(bin)) => {
                if let Err(err) = tty_writer.write_all(&bin).await {
                    log::error!("WS → TTY: Failed to write binary: {}", err);
                    return;
                } else {
                    log::debug!("WS → TTY: Sent binary ({} bytes)", bin.len());
                }
            }

            Ok(AggregatedMessage::Ping(msg)) => {
                if let Err(err) = ws_session.pong(&msg).await {
                    log::error!("WS → WS: Failed to respond to ping: {}", err);
                    return;
                } else {
                    log::debug!("WS → WS: Responded to ping");
                }
            }

            Ok(_) => {
                log::trace!("WS → TTY: Ignored unsupported message");
            }

            Err(err) => {
                log::error!("WS → TTY: Error reading WebSocket message: {}", err);
                return;
            }
        }
    }

    log::info!("WS → SSH: WebSocket stream closed.");
}

async fn relay_messages_from_ssh_to_client(
    mut ws_session: WSSession,
    mut tty_reader: ReadHalf<TcpStream>,
) {
    loop {
        let mut buf = [0u8; 8196];
        match tty_reader.read(&mut buf).await {
            Ok(0) => {
                log::info!("TTY → WS: Reached EOF (client disconnected).");
                break;
            }
            Ok(n) => {
                let binmsg = Bytes::copy_from_slice(&buf[..n]);
                if let Err(err) = ws_session.binary(binmsg).await {
                    log::error!(
                        "TTY → WS: Failed to send binary message ({} bytes): {}",
                        n,
                        err
                    );
                    break;
                } else {
                    log::debug!("TTY → WS: Sent binary ({} bytes)", n);
                }
            }
            Err(err) => {
                log::error!("TTY → WS: Failed to read from TTY session: {}", err);
                break;
            }
        }
    }

    log::info!("TTY → WS: TTY reader loop exited.");
}
