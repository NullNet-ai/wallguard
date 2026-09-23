use crate::netinfo::{service::Protocol, sock::SocketInfo};

use std::{net::SocketAddr, time::Duration};
use tokio::{io::AsyncReadExt, time::timeout};

const SSH_TIMEOUT: Duration = Duration::from_millis(100);

async fn is_ssh_impl(addr: SocketAddr) -> bool {
    let Some((mut stream, _)) = super::connect_probe(addr).await else {
        return false;
    };

    let mut buf = [0u8; 128];
    if let Ok(n) = stream.read(&mut buf).await
        && n > 0
    {
        let s = String::from_utf8_lossy(&buf[..n]);
        return s.starts_with("SSH-");
    }

    false
}

// The timeout bounds the connect as well as the banner read: a listener
// whose accept backlog is full drops SYNs, and an unbounded connect would
// then hang for the kernel's full SYN-retry period on every scan.
async fn is_ssh(addr: SocketAddr) -> bool {
    timeout(SSH_TIMEOUT, is_ssh_impl(addr))
        .await
        .unwrap_or(false)
}

pub(super) async fn filter(sockets: &mut Vec<SocketInfo>) -> Vec<(SocketInfo, Protocol)> {
    let mut services = Vec::new();
    let mut remaining = Vec::with_capacity(sockets.len());

    // Probe every candidate socket concurrently rather than awaiting each one
    // sequentially — otherwise cost scales linearly (up to SSH_TIMEOUT per
    // socket) with the number of open listening ports every scan cycle.
    let mut set = tokio::task::JoinSet::new();
    for socket in sockets.drain(..) {
        set.spawn(async move {
            let matched = matches!(socket.protocol, crate::netinfo::sock::Protocol::Tcp)
                && is_ssh(socket.sockaddr).await;
            (socket, matched)
        });
    }

    while let Some(joined) = set.join_next().await {
        let Ok((socket, matched)) = joined else {
            continue;
        };

        if matched {
            services.push((socket, Protocol::Ssh));
        } else {
            remaining.push(socket);
        }
    }

    *sockets = remaining;
    services
}
