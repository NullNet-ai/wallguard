use super::{Protocol, SocketInfo};
use std::net::SocketAddr;
use tokio::process::Command;

fn parse_sockstat_addr(addr_str: &str, is_ipv6: bool) -> Option<SocketAddr> {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

    let (host, port_str) = addr_str.rsplit_once(':')?;
    let port = port_str.parse::<u16>().ok()?;

    if is_ipv6 {
        let host = if host.starts_with('[') && host.ends_with(']') {
            &host[1..host.len() - 1]
        } else {
            host
        };

        if host == "*" {
            Some(SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::UNSPECIFIED,
                port,
                0,
                0,
            )))
        } else {
            host.parse::<Ipv6Addr>()
                .ok()
                .map(|addr| SocketAddr::V6(SocketAddrV6::new(addr, port, 0, 0)))
        }
    } else {
        if host == "*" {
            Some(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::UNSPECIFIED,
                port,
            )))
        } else {
            host.parse::<Ipv4Addr>()
                .ok()
                .map(|addr| SocketAddr::V4(SocketAddrV4::new(addr, port)))
        }
    }
}

pub(super) async fn get_sockets_info() -> Vec<SocketInfo> {
    let Ok(output) = Command::new("sockstat").args(&["-l"]).output().await else {
        return vec![];
    };

    if !output.status.success() {
        return vec![];
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sockets = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        // Format: USER COMMAND PID FD PROTO LOCAL REMOTE
        let command = parts[1];
        let proto_str = parts[4];
        let addr_str = parts[5];

        let (protocol, is_ipv6) = match proto_str {
            "tcp4" | "tcp46" => (Protocol::TCP, false),
            "tcp6" => (Protocol::TCP, true),
            "udp4" | "udp46" => (Protocol::UDP, false),
            "udp6" => (Protocol::UDP, true),
            _ => continue,
        };

        if let Some(sockaddr) = parse_sockstat_addr(addr_str, is_ipv6) {
            sockets.push(SocketInfo {
                process_name: command.to_string(),
                protocol,
                sockaddr,
            });
        }
    }

    sockets
}
