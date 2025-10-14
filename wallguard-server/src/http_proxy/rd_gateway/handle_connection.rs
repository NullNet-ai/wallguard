use std::sync::Arc;

use crate::{
    http_proxy::rd_gateway::signal_message::SignalMessage, reverse_tunnel::TunnelInstance,
};
use actix_ws::{Message, MessageStream, Session};
use futures_util::StreamExt;
use wallguard_common::{
    protobuf::wallguard_tunnel::client_frame::Message as ClientMessage,
    timestamped_packet::TimestampedPacket,
};
use webrtc::{
    api::{
        APIBuilder,
        interceptor_registry::register_default_interceptors,
        media_engine::{MIME_TYPE_H264, MediaEngine},
    },
    ice_transport::{ice_candidate::RTCIceCandidateInit, ice_server::RTCIceServer},
    interceptor,
    media::Sample,
    peer_connection::{
        RTCPeerConnection, configuration::RTCConfiguration,
        sdp::session_description::RTCSessionDescription,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::track_local_static_sample::TrackLocalStaticSample,
};

pub async fn handle_connection(stream: MessageStream, session: Session, tunnel: TunnelInstance) {
    let mut media_engine = MediaEngine::default();
    let _ = media_engine.register_default_codecs();

    let registry = interceptor::registry::Registry::new();
    let registry = register_default_interceptors(registry, &mut media_engine).unwrap();

    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());

    let sesh = session.clone();
    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let mut sesh = sesh.clone();
        Box::pin(async move {
            if let Some(candidate) = candidate {
                let candidate_json = candidate.to_json().unwrap();

                let msg = SignalMessage::Ice {
                    candidate: serde_json::to_string(&candidate_json).unwrap(),
                };

                let _ = sesh.text(serde_json::to_string(&msg).unwrap()).await;
            }
        })
    }));

    peer_connection.on_peer_connection_state_change(Box::new(move |_| Box::pin(async {})));

    peer_connection.on_ice_connection_state_change(Box::new(move |_| Box::pin(async {})));

    peer_connection.on_data_channel(Box::new(move |data_channel| {
        data_channel.on_open(Box::new(move || Box::pin(async {})));
        data_channel.on_open(Box::new(move || Box::pin(async {})));

        let d2 = data_channel.clone();
        data_channel.on_message(Box::new(move |msg| {
            let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();

            let reversed: String = msg_str.chars().rev().collect();
            let response = format!("Reversed: {}", reversed);
            let d2 = d2.clone();
            Box::pin(async move {
                d2.send_text(response).await.unwrap();
            })
        }));

        Box::pin(async {})
    }));

    let video_track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_H264.to_owned(),
            clock_rate: 90000,
            channels: 0,
            ..Default::default()
        },
        "video".to_string(),
        "main".to_string(),
    ));

    let Ok(rtp_sender) = peer_connection.add_track(video_track.clone()).await else {
        return log::error!("Failed to add video track to the peer connection");
    };

    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        println!("RTCP reader stopped");
    });

    tokio::select! {
        _ = handle_signaling(stream, session, peer_connection.clone()) => {}
        _ = handle_messages_from_remote_desktop(tunnel, video_track) => {}
    }
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
