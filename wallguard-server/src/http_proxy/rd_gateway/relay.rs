use crate::{http_proxy::rd_gateway::webrtc::WebRTCSession, reverse_tunnel::TunnelInstance};
use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt;
use std::time::Duration;
use wallguard_common::protobuf::wallguard_tunnel::client_frame::Message as ClientMessage;

pub(crate) async fn relay(
    webrtc: WebRTCSession,
    tty_tunnel: TunnelInstance,
    msg_stream: MessageStream,
    ws_session: WSSession,
) {
    let stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    tokio::select! {
        _ = relay_messages_from_user_to_client(
            stream,
            tty_tunnel.clone(),
            ws_session.clone(),
            webrtc.clone()
        ) => {
            log::info!("WebRTC → RD relay ended.");
        }
        _ = relay_messages_from_rd_to_client(webrtc, tty_tunnel) => {
            log::info!("RD → WebRTC relay ended.");
        }
    }
}

async fn relay_messages_from_user_to_client(
    mut stream: AggregatedMessageStream,
    _rd_tunnel: TunnelInstance,
    mut ws_session: WSSession,
    webrtc_session: WebRTCSession,
) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(AggregatedMessage::Text(text)) => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
                    log::error!("Received a non-JSON message from RD client");
                    continue;
                };

                let Some(message_type) = json
                    .as_object()
                    .and_then(|obj| obj.get("type"))
                    .and_then(|value| value.as_str())
                else {
                    log::error!("Received a malformed message from RD client");
                    continue;
                };

                match message_type.to_lowercase().as_str() {
                    "candidate" => {
                        let Some(data) = json.as_object().and_then(|obj| obj.get("candidate"))
                        else {
                            log::error!("Received a malformed message from RD client");
                            continue;
                        };

                        let Ok(candidate) = serde_json::from_value(data.clone()) else {
                            log::error!("Received a malformed message from RD client");
                            continue;
                        };

                        if webrtc_session.add_candidate(candidate).await.is_err() {
                            log::error!("Failed to add candidate");
                        }
                    }
                    mt => {
                        log::error!("Unsupported message type {mt}");
                        continue;
                    }
                };
            }

            // Ok(AggregatedMessage::Text(text)) => {
            //     let data_frame = TunnelInstance::make_data_frame(text.as_bytes());
            //     if let Err(err) = rd_tunnel.write(data_frame).await {
            //         log::error!("WS → RD: Failed to write text: {}", err.to_str());
            //         return;
            //     } else {
            //         log::debug!("WS → RD: Sent text ({} bytes)", text.len());
            //     }
            // }

            // Ok(AggregatedMessage::Binary(bin)) => {
            //     let data_frame = TunnelInstance::make_data_frame(&bin);
            //     if let Err(err) = rd_tunnel.write(data_frame).await {
            //         log::error!("WS → RD: Failed to write binary: {}", err.to_str());
            //         return;
            //     } else {
            //         log::debug!("WS → RD: Sent binary ({} bytes)", bin.len());
            //     }
            // }
            Ok(AggregatedMessage::Ping(msg)) => {
                if let Err(err) = ws_session.pong(&msg).await {
                    log::error!("WS → WS: Failed to respond to ping: {err}");
                    return;
                } else {
                    log::debug!("WS → WS: Responded to ping");
                }
            }

            Ok(_) => {
                log::trace!("WS → RD: Ignored unsupported message");
            }

            Err(err) => {
                log::error!("WS → RD: Error reading WebSocket message: {err}");
                return;
            }
        }
    }

    log::info!("WS → RD: WebSocket stream closed.");
}

async fn relay_messages_from_rd_to_client(webrtc: WebRTCSession, rd_tunnel: TunnelInstance) {
    loop {
        let Ok(message) = rd_tunnel.read().await else {
            log::error!("RD → WebRTC: Failed to read from RD session");
            break;
        };

        let Some(message) = message.message else {
            log::info!("RD → WebRTC: Reached EOF (client disconnected).");
            break;
        };

        let ClientMessage::Data(data) = message else {
            log::error!("RD → WebRTC: Unexpected message.");
            break;
        };

        let len = data.data.len();

        if let Err(err) = webrtc.send(data.data, Duration::from_millis(33)).await {
            log::error!("RD → WebRTC: Failed to send sample: {}", err.to_str());
            break;
        } else {
            log::debug!("RD → WebRTC: Sent sample ({len} bytes)");
        }
    }

    log::info!("RD → WS: RD reader loop exited.");
}
