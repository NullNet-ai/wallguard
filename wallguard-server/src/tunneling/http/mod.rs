use crate::datastore::{ServiceInfo, TunnelModel};
use crate::reverse_tunnel::TunnelInstance;
use crate::tunneling::async_io::AsyncIo;
use crate::tunneling::tunnel_common_data::{TunnelCommonData, TunnelCreateError};
use crate::{app_context::AppContext, tunneling::tunnel_common::TunnelCommon};
use async_trait::async_trait;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::pin::Pin;

#[derive(Debug)]
pub struct HttpTunnel {
    data: TunnelCommonData,
    context: AppContext,
}

#[async_trait]
impl TunnelCommon for HttpTunnel {
    async fn create(
        context: AppContext,
        data: TunnelCommonData,
    ) -> Result<Self, TunnelCreateError> {
        Ok(Self { data, context })
    }

    async fn terminate(&self) -> Result<(), Error> {
        todo!()
    }

    async fn request_tunnel(&self) -> Result<TunnelInstance, Error> {
        let instance_id = self
            .context
            .orchestractor
            .get_any_client_instance(&self.data.tunnel_data.device_id)
            .await
            .ok_or("Device not found")
            .handle_err(location!())?
            .lock()
            .await
            .instance_id
            .clone();

        super::command::establish_tunneled_ui(
            &self.context,
            &self.data.tunnel_data.device_id,
            &instance_id,
            &self.data.service_data.protocol,
            &self.data.service_data.address,
            self.data.service_data.port as u32,
        )
        .await
    }

    async fn request_session(&self) -> Result<Pin<Box<dyn AsyncIo + Send>>, Error> {
        Err("http tunnel is state-less").handle_err(location!())
    }

    fn get_service_data(&self) -> &ServiceInfo {
        &self.data.service_data
    }

    fn get_tunnel_data(&self) -> &TunnelModel {
        &self.data.tunnel_data
    }
}
