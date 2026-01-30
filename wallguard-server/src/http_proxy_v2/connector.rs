use pingora::connectors::L4Connect;
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;
use pingora::protocols::l4::stream::Stream;
use tokio::io::BufStream;
use tonic::async_trait;

use crate::app_context::AppContext;
use crate::datastore::ServiceInfo;
use crate::reverse_tunnel::TunnelAdapter;

#[derive(Debug)]
pub struct Connector {
    context: AppContext,
    service: ServiceInfo,
}

impl Connector {
    pub fn new(context: AppContext, service: ServiceInfo) -> Self {
        Self { context, service }
    }
}

#[async_trait]
impl L4Connect for Connector {
    async fn connect(&self, _addr: &SocketAddr) -> Result<Stream> {
        let Some(instance) = self
            .context
            .orchestractor
            .get_any_client_instance(&self.service.device_id)
            .await
        else {
            return Err(Error::new(ErrorType::Custom("Device is offline")));
        };

        let instance_id = instance.lock().await.instance_id.clone();

        let Ok(tunnel) = crate::http_proxy::utilities::tunneling::establish_tunneled_ui(
            &self.context,
            &self.service.device_id,
            &instance_id,
            &self.service.protocol,
            &self.service.address,
            self.service.port.into(),
        )
        .await
        else {
            return Err(Error::new(ErrorType::Custom(
                "Failed to establish a tunnel",
            )));
        };

        if !tunnel.is_authenticated() {
            return Err(Error::new(ErrorType::Custom("Tunnel is not authenticated")));
        }

        let Ok(tunnel_adapter) = TunnelAdapter::try_from(tunnel) else {
            return Err(Error::new(ErrorType::InternalError));

        };

        let s = BufStream::with_capacity(0, 0, tunnel_adapter);


        // Stream::from(tunnel_adapter);
        // Ok(L4Stream::from(socket))

        // Ok(Stream::from(value))

        // BufStream::with_capacity(0, 0, )

        todo!()
    }
}
