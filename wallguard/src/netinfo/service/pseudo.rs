use crate::netinfo::sock::SocketInfo;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::netinfo::service::ServiceInfo;

pub fn filter(_: &[SocketInfo]) -> Vec<ServiceInfo> {
    vec![ServiceInfo {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        protocol: super::Protocol::Tty,
        program: String::from("/wallguard-tty"),
    }]
}
