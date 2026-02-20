use pingora::connectors::L4Connect;
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;
use pingora::protocols::l4::stream::Stream;
use tonic::async_trait;

use crate::app_context::AppContext;
use crate::datastore::ServiceInfo;

#[derive(Debug)]
pub struct Connector {
    context: AppContext,
    tunnel: Tunnel,
}

impl Connector {
    pub fn new(context: AppContext, service: ServiceInfo) -> Self {
        Self { context, service }
    }
}

#[async_trait]
impl L4Connect for Connector {
    async fn connect(&self, _addr: &SocketAddr) -> Result<Stream> {
        log::info!(
            "PROXY -- CONNECTOR: Connecting to service {:?}",
            self.service
        );

        let Some(instance) = self
            .context
            .orchestractor
            .get_any_client_instance(&self.service.device_id)
            .await
        else {
            log::error!("PROXY: - CONNECTOR - Device is offline");
            return Err(Error::new(ErrorType::Custom("Device is offline")));
        };

        let instance_id = instance.lock().await.instance_id.clone();

        let Ok(tunnel) = crate::http_api::utilities::tunneling::establish_tunneled_ui(
            &self.context,
            &self.service.device_id,
            &instance_id,
            &self.service.protocol,
            &self.service.address,
            self.service.port.into(),
        )
        .await
        else {
            log::error!("PROXY: - CONNECTOR - Failed to establish a tunnel");
            return Err(Error::new(ErrorType::Custom(
                "Failed to establish a tunnel",
            )));
        };

        log::info!("PROXY -- CONNECTOR: DONE");

        Ok(Stream::from(tunnel.take_stream()))
    }
}
