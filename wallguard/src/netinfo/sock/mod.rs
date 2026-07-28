use std::net::SocketAddr;

use listeners::{Listener, Protocol as ListenersProtocol, SocketState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketInfo {
    pub process_name: String,
    pub protocol: Protocol,
    pub sockaddr: SocketAddr,
}

fn into_socket_info(listener: Listener) -> Option<SocketInfo> {
    let protocol = match listener.protocol {
        // UDP has no listening state of its own, so every UDP socket is kept
        // (matches the previous per-platform implementations' behavior).
        ListenersProtocol::UDP => Protocol::Udp,
        // Only TCP sockets actively accepting connections are candidate services.
        ListenersProtocol::TCP if listener.state == SocketState::Listen => Protocol::Tcp,
        ListenersProtocol::TCP => return None,
    };

    Some(SocketInfo {
        process_name: listener.process.name,
        protocol,
        sockaddr: listener.socket,
    })
}

pub async fn get_sockets_info() -> Vec<SocketInfo> {
    tokio::task::spawn_blocking(|| {
        listeners::get_all()
            .unwrap_or_default()
            .into_iter()
            .filter_map(into_socket_info)
            .collect()
    })
    .await
    .unwrap_or_default()
}
