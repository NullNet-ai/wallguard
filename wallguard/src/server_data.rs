use crate::arguments::Arguments;
use nullnet_liberror::{location, ErrorHandler, Location};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ServerData {
    pub(crate) grpc_addr: SocketAddr,
    pub(crate) tunn_addr: SocketAddr,
}

impl TryFrom<&Arguments> for ServerData {
    type Error = nullnet_liberror::Error;

    fn try_from(arguments: &Arguments) -> Result<Self, Self::Error> {
        let grpc_addr = format!(
            "{}:{}",
            arguments.control_channel_host, arguments.control_channel_port
        );
        let tunn_addr = format!("{}:{}", arguments.tunnel_host, arguments.tunnel_port);

        let grpc_addr = grpc_addr.parse().handle_err(location!())?;
        let tunn_addr = tunn_addr.parse().handle_err(location!())?;

        Ok(Self {
            grpc_addr,
            tunn_addr,
        })
    }
}
