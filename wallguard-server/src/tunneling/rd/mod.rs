use nullnet_liberror::Error;

use crate::{
    app_context::AppContext,
    datastore::TunnelStatus,
    reverse_tunnel::TunnelInstance,
    tunneling::{
        rd::session::{SessionDataReceiver, SessionDataSender},
        tunnel_common::{TunnelCommonData, TunnelCreateError},
    },
};
use std::sync::Arc;

mod internal_relay;
pub mod session;

#[derive(Debug, Clone)]
pub struct RdTunnel {
    pub data: TunnelCommonData,
    context: Arc<AppContext>,
    session: Arc<session::Session>,
}

impl RdTunnel {
    pub async fn new(
        context: Arc<AppContext>,
        data: TunnelCommonData,
    ) -> Result<Self, TunnelCreateError> {
        let tunnel_instance = Self::request_tunnel_stream(&context, &data.tunnel_data.device_id)
            .await
            .map_err(|_| TunnelCreateError::CantEstablishATunnel)?;

        let session = session::Session::new(
            context.clone(),
            tunnel_instance,
            data.tunnel_data.id.clone(),
        )
        .await
        .map_err(|_| TunnelCreateError::CantEstablishATunnel)?;

        Ok(Self {
            data,
            context,
            session: Arc::new(session),
        })
    }

    async fn request_tunnel_stream(
        context: &AppContext,
        device_id: &str,
    ) -> Result<TunnelInstance, Error> {
        use super::command::establish_tunneled_rd;
        establish_tunneled_rd(context, device_id).await
    }

    pub fn get_data_send_channel(&self) -> SessionDataSender {
        self.session.get_data_send_channel()
    }

    pub fn get_data_recv_channel(&self) -> SessionDataReceiver {
        self.session.get_data_recv_channel()
    }

    pub async fn terminate(&self) -> Result<(), Error> {
        self.session.signal().await;
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

    pub fn has_active_viewers(&self) -> bool {
        self.session.has_active_viewers()
    }
}
