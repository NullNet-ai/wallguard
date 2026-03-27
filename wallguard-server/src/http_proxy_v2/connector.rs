use pingora::connectors::L4Connect;
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;
use pingora::protocols::l4::stream::Stream;
use tonic::async_trait;

use crate::tunneling::tunnel_common::WallguardTunnel;

#[derive(Debug)]
pub struct Connector {
    tunnel: WallguardTunnel,
}

impl Connector {
    pub fn new(tunnel: WallguardTunnel) -> Self {
        Self { tunnel }
    }
}

#[async_trait]
impl L4Connect for Connector {
    async fn connect(&self, _: &SocketAddr) -> Result<Stream> {
        let WallguardTunnel::Http(tunnel) = self.tunnel.clone() else {
            return Err(Error::new(ErrorType::Custom(
                "can't connect, wront tunnel type",
            )));
        };

        let Ok(tunnel_stream) = tunnel.lock().await.request_stream().await else {
            return Err(Error::new(ErrorType::Custom(
                "can't connect, failed to establish a tunnel",
            )));
        };

        Ok(Stream::from(tunnel_stream.take_stream()))
    }
}
