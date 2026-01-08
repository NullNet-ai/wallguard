use crate::netinfo::sock::{IpVersion, Protocol};

mod sock;

pub struct ServiceInfo {}

pub async fn perform_service_discovery() -> Vec<ServiceInfo> {
    let sockets = sock::get_sockets_info().await;

    for socket in sockets.iter() {
        if matches!(socket.protocol, Protocol::TCP) && matches!(socket.ip_version, IpVersion::V4) {
            println!("{:?}", socket);
        }
    }

    vec![]
}
