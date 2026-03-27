use crate::netinfo::sock::SocketInfo;
use std::net::SocketAddr;
use wallguard_common::protobuf::wallguard_service::{
    ServiceInfo as ServiceInfoGrpc, ServiceProtocol as ProtocolGrpc,
};

mod http;
mod pseudo;
mod ssh;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Protocol {
    Http,
    Https,
    Ssh,
    Tty,
}

#[derive(Debug)]
pub struct ServiceInfo {
    addr: SocketAddr,
    protocol: Protocol,
    program: String,
}

impl From<ServiceInfo> for ServiceInfoGrpc {
    fn from(val: ServiceInfo) -> Self {
        ServiceInfoGrpc {
            protocol: match val.protocol {
                Protocol::Http => ProtocolGrpc::Http.into(),
                Protocol::Https => ProtocolGrpc::Https.into(),
                Protocol::Ssh => ProtocolGrpc::Ssh.into(),
                Protocol::Tty => ProtocolGrpc::Tty.into(),
            },
            program: val.program,
            address: val.addr.ip().to_string(),
            port: val.addr.port().into(),
        }
    }
}

pub async fn gather_info(mut sockets: Vec<SocketInfo>) -> Vec<ServiceInfo> {
    let mut retval = vec![];

    retval.extend(http::filter(&mut sockets).await);
    retval.extend(ssh::filter(&mut sockets).await);
    retval.extend(pseudo::filter(&mut sockets));

    retval
}
