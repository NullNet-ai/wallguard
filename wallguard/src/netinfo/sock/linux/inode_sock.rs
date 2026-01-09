use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use tokio::{
    fs,
    io::{self, AsyncBufReadExt},
};

use crate::netinfo::sock::{IpVersion, Protocol};

async fn parse_proc_net(
    path: &str,
    proto: Protocol,
    version: IpVersion,
) -> io::Result<HashMap<u64, (SocketAddr, Protocol)>> {
    let file = fs::File::open(path).await?;
    let reader = io::BufReader::new(file);
    let mut map = HashMap::new();

    let mut lines = reader.lines();

    // Skip the header line
    lines.next_line().await?;

    while let Some(line) = lines.next_line().await? {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 10 {
            continue;
        }

        if let Protocol::TCP = proto {
            // Only include TCP if state == "0A" (LISTEN)
            let state_hex = cols[3];
            if state_hex != "0A" {
                continue;
            }
        }

        let local = cols[1];
        let inode: u64 = cols[9].parse().unwrap_or(0);

        if let Some(colon) = local.find(':') {
            let (addr_hex, port_hex) = local.split_at(colon);
            let addr_hex = &addr_hex[..addr_hex.len()];
            let port_hex = &port_hex[1..];

            if let Ok(port) = u16::from_str_radix(port_hex, 16) {
                let sockaddr = match version {
                    IpVersion::V4 => {
                        if addr_hex.len() != 8 {
                            continue;
                        };

                        let ip = u32::from_str_radix(addr_hex, 16).unwrap();
                        let a = ((ip >> 00) & 0xff) as u8;
                        let b = ((ip >> 08) & 0xff) as u8;
                        let c = ((ip >> 16) & 0xff) as u8;
                        let d = ((ip >> 24) & 0xff) as u8;

                        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), port))
                    }
                    IpVersion::V6 => {
                        if addr_hex.len() != 32 {
                            continue;
                        };

                        let mut bytes = [0u8; 16];
                        for i in 0..16 {
                            let byte = u8::from_str_radix(&addr_hex[i * 2..i * 2 + 2], 16).unwrap();
                            bytes[15 - i] = byte;
                        }

                        SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from_octets(bytes), port, 0, 0))
                    }
                };

                map.insert(inode, (sockaddr, proto));
            }
        }
    }
    Ok(map)
}

pub(super) async fn build_inode_sock_map() -> HashMap<u64, (SocketAddr, Protocol)> {
    let mut inode_map = HashMap::new();

    if let Ok(tcp_map) = parse_proc_net("/proc/net/tcp", Protocol::TCP, IpVersion::V4).await {
        inode_map.extend(tcp_map);
    }

    if let Ok(tcp6_map) = parse_proc_net("/proc/net/tcp6", Protocol::TCP, IpVersion::V6).await {
        inode_map.extend(tcp6_map);
    }

    if let Ok(udp_map) = parse_proc_net("/proc/net/udp", Protocol::UDP, IpVersion::V4).await {
        inode_map.extend(udp_map);
    }

    if let Ok(udp6_map) = parse_proc_net("/proc/net/udp6", Protocol::UDP, IpVersion::V6).await {
        inode_map.extend(udp6_map);
    }

    inode_map
}
