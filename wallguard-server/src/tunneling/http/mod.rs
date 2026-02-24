use std::sync::Arc;

use crate::app_context::AppContext;
use crate::datastore::TunnelStatus;
use crate::reverse_tunnel::TunnelInstance;
use crate::tunneling::tunnel_common::TunnelCommonData;
use nullnet_liberror::Error;

#[derive(Debug, Clone)]
pub struct HttpTunnel {
    pub data: TunnelCommonData,
    context: Arc<AppContext>,
}

impl HttpTunnel {
    pub fn new(context: Arc<AppContext>, data: TunnelCommonData) -> Self {
        Self { context, data }
    }

    pub async fn request_stream(&self) -> Result<TunnelInstance, Error> {
        super::command::establish_tunneled_ui(
            &self.context,
            &self.data.tunnel_data.device_id,
            &self.data.service_data.protocol,
            &self.data.service_data.address,
            self.data.service_data.port as u32,
        )
        .await
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
