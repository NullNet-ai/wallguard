use crate::netinfo::{
    service::{Protocol, ServiceInfo},
    sock::SocketInfo,
};

use std::{net::SocketAddr, time::Duration};
use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

const SSH_TIMEOUT: Duration = Duration::from_millis(100);

async fn is_ssh(addr: SocketAddr) -> bool {
    let Ok(mut stream) = TcpStream::connect(addr).await else {
        return false;
    };

    let mut buf = [0u8; 128];
    if let Ok(Ok(n)) = timeout(SSH_TIMEOUT, stream.read(&mut buf)).await
        && n > 0
    {
        let s = String::from_utf8_lossy(&buf[..n]);
        return s.starts_with("SSH-");
    }

    false
}

pub(super) async fn filter(sockets: &[SocketInfo]) -> Vec<ServiceInfo> {
    let tcp_sockets = sockets
        .iter()
        .filter(|s| matches!(s.protocol, crate::netinfo::sock::Protocol::Tcp));

    let mut services = Vec::new();

    for socket in tcp_sockets {
        if is_ssh(socket.sockaddr).await {
            services.push(ServiceInfo {
                addr: socket.sockaddr,
                protocol: Protocol::Ssh,
                program: socket.process_name.clone(),
            });
        }
    }

    services
}
