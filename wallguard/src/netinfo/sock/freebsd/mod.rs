mod proc_snap;
mod tcp;
mod udp;

use proc_snap::snapshot_processes;
use tcp::{tcp_sockets, tcp6_sockets};
use udp::{udp_sockets, udp6_sockets};

use super::{Protocol, SocketInfo};

pub(super) async fn  get_sockets_info() -> Vec<SocketInfo> {
    let sockets = [
        (
            tcp_sockets().await.unwrap_or_default(),
            Protocol::TCP,
        ),
        (
            tcp6_sockets().await.unwrap_or_default(),
            Protocol::TCP,
        ),
        (
            udp_sockets().await.unwrap_or_default(),
            Protocol::UDP,
        ),
        (
            udp6_sockets().await.unwrap_or_default(),
            Protocol::UDP,
        ),
    ];

    let snapshot = snapshot_processes().await.unwrap_or_default();

    sockets
        .iter()
        .flat_map(|(socks, protocol)| {
            socks.iter().filter_map(|(sockaddr, pid)| {
                snapshot.get(pid).map(|proc_name| SocketInfo {
                    process_name: proc_name.into(),
                    protocol: *protocol,
                    sockaddr: *sockaddr,
                })
            })
        })
        .collect()
}
