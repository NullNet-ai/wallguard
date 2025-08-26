use crate::reverse_tunnel::{ListenersMap, TunnelInstance, tunnel_token::TokenHash};
use wallguard_common::protobuf::wallguard_tunnel::{
    ServerFrame, VerdictFrame, client_frame::Message as ClientMessage,
    server_frame::Message as ServerMessage,
};
pub struct TunnelAuthenticationTask {
    tunnel: TunnelInstance,
    listeners: ListenersMap,
}

impl TunnelAuthenticationTask {
    pub fn new(tunnel: TunnelInstance, listeners: ListenersMap) -> Self {
        Self { tunnel, listeners }
    }

    pub async fn authenticate(mut self) {
        let Ok(message) = self.tunnel.read().await else {
            log::error!("TunnelAuthenticationTask: Failed to read authentication message");
            return;
        };

        let Some(message) = message.message else {
            log::error!("TunnelAuthenticationTask: Client has sent an empty message, aborting ...");
            return;
        };

        let ClientMessage::Authentication(auth_frame) = message else {
            log::error!("TunnelAuthenticationTask: Received unexpected message, aboring");
            return;
        };

        let Ok(token_hash) = TokenHash::try_from(auth_frame.token) else {
            log::error!("TunnelAuthenticationTask: Wrong token format");
            return;
        };

        match self.listeners.lock().await.remove(&token_hash) {
            Some(channel) => {
                self.tunnel.authenticated = true;
                if channel.send(self.tunnel).is_err() {
                    log::error!(
                        "TunnelAuthenticationTask: Failed to send tunnel instance to listener"
                    )
                } else {
                    log::info!("TunnelAuthenticationTask: Successfully authenticated a tunnel");
                }
            }
            None => {
                log::warn!("TunnelAuthenticationTask: Received wrong token");
                let _ = self
                    .tunnel
                    .write(ServerFrame {
                        message: Some(ServerMessage::Verdict(VerdictFrame { allowed: false })),
                    })
                    .await;
            }
        }
    }
}
