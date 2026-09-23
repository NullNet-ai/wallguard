use crate::netinfo::sock::SocketInfo;
use crate::utilities::net::local_dial_targets;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use wallguard_common::protobuf::wallguard_service::{
    ServiceInfo as ServiceInfoGrpc, ServiceProtocol as ProtocolGrpc,
};

mod http;
mod pseudo;
mod pseudo_rd;
mod ssh;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Protocol {
    Http,
    Https,
    Ssh,
    Tty,
    RemoteDesktop,
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
                Protocol::RemoteDesktop => ProtocolGrpc::Rd.into(),
            },
            program: val.program,
            address: val.addr.ip().to_string(),
            port: val.addr.port().into(),
        }
    }
}

/// Opens a probe connection to the listener bound to `addr`, returning the
/// stream and the address it actually reached.
pub(super) async fn connect_probe(addr: SocketAddr) -> Option<(TcpStream, SocketAddr)> {
    for target in local_dial_targets(addr) {
        if let Ok(stream) = TcpStream::connect(target).await {
            return Some((stream, target));
        }
    }

    None
}

/// Remembers the probe verdict for every listener seen so far, so that each
/// listener is probed once instead of on every scan.
///
/// Probing means opening TCP connections to every listening port on the
/// host, and some listeners never `accept()` them. Docker's port
/// reservation sockets (owned by `dockerd` when the userland proxy is off)
/// are the known case: a probe to one of those completes the handshake in
/// the kernel, then sits in the accept queue with its unread payload in
/// CLOSE_WAIT until the listener is closed. Re-probing each listener every
/// scan grew those queues by thousands of sockets per port and exhausted
/// the host's TCP memory.
///
/// A verdict is kept while the listener exists (same process name, protocol
/// and address) and forgotten once it disappears, so a restarted or
/// replaced service is probed afresh. The tradeoff is that a service which
/// answered its single probe with a non-2xx status stays unreported until
/// it restarts.
#[derive(Debug, Default)]
pub struct ServiceScanner {
    verdicts: HashMap<SocketInfo, Option<Protocol>>,
}

impl ServiceScanner {
    pub async fn gather_info(&mut self, sockets: Vec<SocketInfo>) -> Vec<ServiceInfo> {
        let current: HashSet<&SocketInfo> = sockets.iter().collect();
        self.verdicts.retain(|socket, _| current.contains(socket));

        let mut unprobed: Vec<SocketInfo> = sockets
            .iter()
            .filter(|socket| !self.verdicts.contains_key(*socket))
            .cloned()
            .collect();

        if !unprobed.is_empty() {
            let detected = http::filter(&mut unprobed)
                .await
                .into_iter()
                .chain(ssh::filter(&mut unprobed).await);

            for (socket, protocol) in detected {
                self.verdicts.insert(socket, Some(protocol));
            }

            // Whatever neither filter claimed is not a service we report.
            for socket in unprobed {
                self.verdicts.insert(socket, None);
            }
        }

        let mut retval: Vec<ServiceInfo> = sockets
            .iter()
            .filter_map(|socket| {
                let protocol = (*self.verdicts.get(socket)?)?;
                Some(ServiceInfo {
                    addr: socket.sockaddr,
                    protocol,
                    program: socket.process_name.clone(),
                })
            })
            .collect();

        let mut sockets = sockets;
        retval.extend(pseudo::filter(&mut sockets));

        // pseudo_rd performs its own live check (tries Enigo::new) so it
        // naturally reports nothing when no user session is active.
        retval.extend(pseudo_rd::filter(&mut sockets));

        retval
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netinfo::sock::Protocol as SockProtocol;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::net::TcpListener;

    /// A listener that accepts connections, counts them, and never answers,
    /// so every probe fails and the socket gets a negative verdict.
    async fn counting_listener() -> (SocketInfo, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let accepted = Arc::new(AtomicUsize::new(0));

        let counter = accepted.clone();
        tokio::spawn(async move {
            let mut held = Vec::new();
            while let Ok((stream, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                held.push(stream);
            }
        });

        let socket = SocketInfo {
            process_name: "test".into(),
            protocol: SockProtocol::Tcp,
            sockaddr: addr,
        };

        (socket, accepted)
    }

    #[tokio::test]
    async fn listener_is_probed_only_once() {
        let (socket, accepted) = counting_listener().await;
        let mut scanner = ServiceScanner::default();

        scanner.gather_info(vec![socket.clone()]).await;
        let after_first = accepted.load(Ordering::SeqCst);
        assert!(after_first > 0, "first scan should probe the listener");

        scanner.gather_info(vec![socket.clone()]).await;
        scanner.gather_info(vec![socket]).await;
        assert_eq!(accepted.load(Ordering::SeqCst), after_first);
    }

    #[tokio::test]
    async fn vanished_listener_is_probed_again_when_it_returns() {
        let (socket, accepted) = counting_listener().await;
        let mut scanner = ServiceScanner::default();

        scanner.gather_info(vec![socket.clone()]).await;
        let after_first = accepted.load(Ordering::SeqCst);

        scanner.gather_info(vec![]).await;
        scanner.gather_info(vec![socket]).await;
        assert!(accepted.load(Ordering::SeqCst) > after_first);
    }
}
