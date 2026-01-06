use crate::arguments::Arguments;
use nullnet_liberror::{ErrorHandler, Location, location};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

#[derive(Debug, Clone)]
pub struct ServerData {
    pub(crate) grpc_addr: SocketAddr,
}

impl TryFrom<&Arguments> for ServerData {
    type Error = nullnet_liberror::Error;

    fn try_from(arguments: &Arguments) -> Result<Self, Self::Error> {
        let grpc_addr = match arguments.control_channel_url.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                let addrs_iter = arguments
                    .control_channel_url
                    .to_socket_addrs()
                    .handle_err(location!())?;

                let mut ipv6 = None;
                let mut ipv4 = None;

                for addr in addrs_iter {
                    match addr.ip() {
                        IpAddr::V4(_) => {
                            ipv4 = Some(addr);
                            break;
                        }
                        IpAddr::V6(_) => {
                            ipv6 = Some(addr);
                        }
                    }
                }

                ipv4.or(ipv6)
                    .ok_or("Failed to resolve address")
                    .handle_err(location!())?
            }
        };

        Ok(Self { grpc_addr })
    }
}
