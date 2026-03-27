use crate::{
    app_context::AppContext, datastore::TunnelStatus, tunneling::tunnel_common::TunnelCommonData,
};
use actix_ws::Session;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::sync::Arc;

pub mod peer;
mod signal;

#[derive(Debug, Clone)]
pub struct RemoteDesktopTunnel {
    pub data: TunnelCommonData,
    context: Arc<AppContext>,
}

impl RemoteDesktopTunnel {
    pub fn new(context: Arc<AppContext>, data: TunnelCommonData) -> Self {
        Self { context, data }
    }

    pub async fn request_peer(&self, session: Session) -> Result<peer::RdPeerConnection, Error> {
        let tunnel =
            super::command::establish_tunneled_rd(&self.context, &self.data.tunnel_data.device_id)
                .await?;

        peer::RdPeerConnection::new(session, tunnel)
            .await
            .ok_or("Failed to establish peer connection")
            .handle_err(location!())
    }

    pub async fn terminate(&self) -> Result<(), Error> {
        let token = self.context.sysdev_token_provider.get().await?;

        self.context
            .datastore
            .update_tunnel_status(
                &token.jwt,
                &self.data.tunnel_data.id,
                TunnelStatus::Terminated,
                false,
            )
            .await
    }
}
