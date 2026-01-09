use crate::netinfo::sock::SocketInfo;
use std::net::SocketAddr;

mod http;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Protocol {
    Http,
    Https,
}

#[derive(Debug)]
pub struct ServiceInfo {
    addr: SocketAddr,
    protocol: Protocol,
    program: String,
}

pub async fn gather_info(sockets: &[SocketInfo]) -> Vec<ServiceInfo> {
    let mut retval = vec![];

    retval.extend(http::filter(sockets).await);

    retval
}
