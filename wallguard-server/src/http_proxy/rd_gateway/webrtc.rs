use actix_ws::{Message as WSMessage, MessageStream, Session as WSSession};
use futures_util::StreamExt;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use webrtc::api::APIBuilder;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice::network_type::NetworkType;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::media::Sample;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::policy::ice_transport_policy::RTCIceTransportPolicy;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

#[derive(Debug, Serialize, Deserialize)]
struct WebRTCSignal {
    sdp: String,
}

#[derive(Debug, Clone)]
pub struct WebRTCSession {
    video_track: Arc<TrackLocalStaticSample>,
    connection: Arc<RTCPeerConnection>,
}

impl WebRTCSession {
    pub async fn establish(
        ws_stream: &mut MessageStream,
        session: &mut WSSession,
    ) -> Result<Self, Error> {
        let mut setting_engine = SettingEngine::default();
        setting_engine.set_network_types(vec![NetworkType::Udp4, NetworkType::Tcp4]);

        let api = APIBuilder::new()
            .with_setting_engine(setting_engine)
            .build();
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".into()],
                ..Default::default()
            }],
            ice_transport_policy: RTCIceTransportPolicy::All,
            ..Default::default()
        };
        let connection = api
            .new_peer_connection(config)
            .await
            .handle_err(location!())?;
        let connection = Arc::new(connection);

        let Some(message) = ws_stream.next().await else {
            Err("WebRTCSession: Unexpected end of stream").handle_err(location!())?
        };

        let message = match message {
            Ok(message) => message,
            Err(err) => Err(format!("WebRTCSession: websocket stream error {err}"))
                .handle_err(location!())?,
        };

        let WSMessage::Text(message) = message else {
            Err("WebRTCSession: Unexpected message").handle_err(location!())?
        };

        let signal = serde_json::from_str::<WebRTCSignal>(&message).handle_err(location!())?;
        let offer = RTCSessionDescription::offer(signal.sdp).handle_err(location!())?;

        connection
            .set_remote_description(offer)
            .await
            .handle_err(location!())?;

        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: "video/h264".to_string(),
                clock_rate: 90000,
                channels: 0,
                ..Default::default()
            },
            // @TODO
            "video".to_string(),
            "main".to_string(),
        ));

        {
            let session = session.clone();
            connection.on_ice_candidate(Box::new(move |candidate| {
                let mut session = session.clone();

                Box::pin(async move {
                    if let Some(candidate) = candidate {
                        let json = serde_json::json!({
                            "type": "candidate",
                            "candidate": candidate.to_json().unwrap()
                        });

                        log::info!("New ICE candidate: {}", candidate.address);

                        let _ = session.text(json.to_string()).await;
                    } else {
                        log::info!("ICE gathering complete");
                    }
                })
            }));
        }

        connection
            .add_track(video_track.clone())
            .await
            .handle_err(location!())?;

        let answer = connection
            .create_answer(None)
            .await
            .handle_err(location!())?;

        connection
            .set_local_description(answer.clone())
            .await
            .handle_err(location!())?;

        let json = json!({
            "type": answer.sdp_type.to_string(),
            "sdp": answer.sdp
        });

        session
            .text(json.to_string())
            .await
            .handle_err(location!())?;

        Ok(Self {
            video_track,
            connection,
        })
    }

    pub async fn send(&self, data: Vec<u8>, duration: Duration) -> Result<(), Error> {
        let sample = Sample {
            data: data.into(),
            duration,
            ..Default::default()
        };

        self.video_track
            .write_sample(&sample)
            .await
            .handle_err(location!())
    }

    pub async fn add_candidate(&self, candidate: RTCIceCandidateInit) -> Result<(), Error> {
        self.connection
            .add_ice_candidate(candidate)
            .await
            .handle_err(location!())
    }
}
