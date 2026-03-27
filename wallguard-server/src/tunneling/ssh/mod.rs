use nullnet_liberror::Error;

use crate::{
    app_context::AppContext,
    datastore::TunnelStatus,
    reverse_tunnel::TunnelInstance,
    tunneling::{
        ssh::{
            session::{SessionDataReceiver, SessionDataSender},
            ssh_data::SshData,
        },
        tunnel_common::{TunnelCommonData, TunnelCreateError},
    },
};
use std::sync::Arc;

mod handler;
mod internal_relay;
mod session;
mod ssh_data;

#[derive(Debug, Clone)]
pub struct SshTunnel {
    pub data: TunnelCommonData,
    context: Arc<AppContext>,
    session: Arc<session::Session>,
}

impl SshTunnel {
    pub async fn new(
        context: Arc<AppContext>,
        data: TunnelCommonData,
    ) -> Result<Self, TunnelCreateError> {
        let ssh_data = SshData::generate(String::from("root"))
            .await
            .map_err(|_| TunnelCreateError::SshKeygenError)?;

        let tunnel_instance =
            Self::request_tunnel_stream(&context, &data.tunnel_data.device_id, &ssh_data)
                .await
                .map_err(|_| TunnelCreateError::CantEstablishATunnel)?;

        let session = session::Session::new(
            context.clone(),
            tunnel_instance,
            &ssh_data,
            data.tunnel_data.id.clone(),
        )
        .await
        .map_err(|_| TunnelCreateError::SshSessionFailed)?;

        Ok(Self {
            data,
            context,
            session: Arc::new(session),
        })
    }

    async fn request_tunnel_stream(
        context: &AppContext,
        device_id: &str,
        data: &SshData,
    ) -> Result<TunnelInstance, Error> {
        use super::command::establish_tunneled_ssh;

        establish_tunneled_ssh(context, device_id, &data.public_key, &data.username).await
    }

    pub fn get_data_send_channel(&self) -> SessionDataSender {
        self.session.get_data_send_channel()
    }

    pub fn get_data_recv_channel(&self) -> SessionDataReceiver {
        self.session.get_data_recv_channel()
    }

    pub async fn get_memory_snaphot(&self) -> Vec<u8> {
        self.session.get_memory_snaphot().await
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

    pub fn has_active_terminals(&self) -> bool {
        self.session.has_active_terminals()
    }
}
