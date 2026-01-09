mod proc_snap;
mod tcp;
mod udp;

use proc_snap::snapshot_processes;
use tcp::{tcp_sockets, tcp6_sockets};
use udp::{udp_sockets, udp6_sockets};

use super::{IpVersion, Protocol, SocketInfo};

pub(super) fn get_sockets_info() -> Vec<SocketInfo> {
    let sockets = [
        (
            tcp_sockets().unwrap_or_default(),
            Protocol::TCP,
            IpVersion::V4,
        ),
        (
            tcp6_sockets().unwrap_or_default(),
            Protocol::TCP,
            IpVersion::V6,
        ),
        (
            udp_sockets().unwrap_or_default(),
            Protocol::UDP,
            IpVersion::V4,
        ),
        (
            udp6_sockets().unwrap_or_default(),
            Protocol::UDP,
            IpVersion::V6,
        ),
    ];

    let snapshot = snapshot_processes().unwrap_or_default();

    sockets
        .iter()
        .flat_map(|(socks, protocol, ip_version)| {
            socks.iter().filter_map(|(addr, pid)| {
                snapshot.get(pid).map(|proc_name| SocketInfo {
                    pid: *pid,
                    process_name: proc_name.into(),
                    protocol: *protocol,
                    ip_version: *ip_version,
                    local_addr: addr.ip(),
                    local_port: addr.port(),
                })
            })
        })
        .collect()
}
