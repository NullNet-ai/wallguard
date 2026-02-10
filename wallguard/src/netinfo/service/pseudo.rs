use std::net::{IpAddrv4, SocketAddr};

use crate::netinfo::service::ServiceInfo;

pub fn filter(_: &[SocketInfo]) -> Vec<ServiceInfo> {
    let mut retval = Vec::new();

    retval.push(ServiceInfo {
        addr: SocketAddr::new(IpAddrv4::new(0, 0, 0, 0), 0),
        protocol: todo!(),
        program: String::from("/wallguard-tty"),
    });

    retval
}
