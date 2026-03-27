use std::sync::Arc;

use crate::{app_context::AppContext, datastore::TunnelStatus, tunneling::{rd::RemoteDesktopTunnel, ssh::SshTunnel}};
use actix_ws::{AggregatedMessage, AggregatedMessageStream, MessageStream, Session as WSSession};
use futures_util::StreamExt as _;
use prost::bytes::Bytes;
use tokio::sync::Mutex;
use wallguard_common::timestamped_packet::TimestampedPacket;
use webrtc::{media::Sample, peer_connection::RTCPeerConnection, srtp::session::Session};

pub async fn websocket_relay(
    stream: MessageStream,
    ws_session: WSSession,
    rd_tunnel: Arc<Mutex<RemoteDesktopTunnel>>,
    context: Arc<AppContext>,
) {

}


async fn handle_signaling(
    mut stream: MessageStream,
    mut session: Session,
    connection: Arc<RTCPeerConnection>,
) {
    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                if let Message::Text(text) = msg
                    && let Ok(signal) = serde_json::from_str::<SignalMessage>(text.as_ref())
                {
                    match signal {
                        SignalMessage::Offer { sdp } => {
                            let offer = RTCSessionDescription::offer(sdp).unwrap();
                            connection.set_remote_description(offer).await.unwrap();

                            let answer = connection.create_answer(None).await.unwrap();
                            connection
                                .set_local_description(answer.clone())
                                .await
                                .unwrap();

                            let msg = SignalMessage::Answer { sdp: answer.sdp };
                            session
                                .text(serde_json::to_string(&msg).unwrap())
                                .await
                                .unwrap();
                        }
                        SignalMessage::Ice { candidate } => {
                            let candidate_init: RTCIceCandidateInit =
                                serde_json::from_str(&candidate).unwrap();
                            connection.add_ice_candidate(candidate_init).await.unwrap();
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                log::error!("WebSocket error: {e}");
                break;
            }
        }
    }
}

async fn handle_messages_from_remote_desktop(
    tunnel: TunnelInstance,
    track: Arc<TrackLocalStaticSample>,
) {
    loop {
        let Ok(message) = tunnel.read().await else {
            log::error!("RD → WebRTC: Failed to read from RD tunnel");
            break;
        };

        let Some(message) = message.message else {
            log::info!("RD → WebRTC: Reached EOF (client disconnected).");
            break;
        };

        let ClientMessage::Data(frame) = message else {
            log::error!("RD → WebRTC: Unexpected message.");
            break;
        };

        let Ok(packet) = TimestampedPacket::from_bytes(&frame.data) else {
            log::error!("Failed to deserialize RD tunnel packet");
            continue;
        };

        let sample = Sample {
            data: packet.data.into(),
            duration: packet.duration,
            ..Default::default()
        };

        if let Err(err) = track.write_sample(&sample).await {
            log::error!("RD → WebRTC: Failed to send sample: {err}");
        } else {
            let len = sample.data.len();
            log::debug!("RD → WebRTC: Sent sample ({len} bytes)");
        }
    }

    log::info!("RD → WS: RD reader loop exited.");
}
