use crate::{reverse_tunnel::TunnelInstance, tunneling::rd::signal::SignalMessage};
use actix_ws::Session;
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::Mutex};
use webrtc::{
    api::{
        APIBuilder,
        interceptor_registry::register_default_interceptors,
        media_engine::{MIME_TYPE_H264, MediaEngine},
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor,
    peer_connection::{RTCPeerConnection, configuration::RTCConfiguration},
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::track_local_static_sample::TrackLocalStaticSample,
};

pub struct RdPeerConnection {
    inner: Arc<RTCPeerConnection>,
    video_track: Arc<TrackLocalStaticSample>,
}

impl RdPeerConnection {
    pub async fn new(session: Session, tunnel: TunnelInstance) -> Option<Self> {
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

        let inner = Arc::new(api.new_peer_connection(config).await.unwrap());

        Self::register_callbacks(&inner, session, tunnel);

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

        if inner.add_track(video_track.clone()).await.is_err() {
            log::error!("Failed to add video track to the peer connection");
            return None;
        }

        Some(Self { inner, video_track })
    }

    fn register_callbacks(
        inner: &Arc<RTCPeerConnection>,
        session: Session,
        tunnel: TunnelInstance,
    ) {
        let sesh = session.clone();
        inner.on_ice_candidate(Box::new(move |candidate| {
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

        inner.on_peer_connection_state_change(Box::new(move |_| Box::pin(async {})));
        inner.on_ice_connection_state_change(Box::new(move |_| Box::pin(async {})));

        let a_tunnel = Arc::new(Mutex::new(tunnel));
        inner.on_data_channel(Box::new(move |data_channel| {
            data_channel.on_open(Box::new(move || Box::pin(async {})));

            let a_tunnel = a_tunnel.clone();
            data_channel.on_message(Box::new(move |msg| {
                let a_tunnel = a_tunnel.clone();
                Box::pin(async move {
                    let _ = a_tunnel
                        .lock()
                        .await
                        .write_all(msg.data.iter().as_slice())
                        .await;
                })
            }));

            Box::pin(async {})
        }));
    }

    async fn close(&self) {
        let _ = self.inner.close().await;
    }
}
