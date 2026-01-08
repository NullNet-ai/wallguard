use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use tokio::fs;
use tokio::io::{self, AsyncBufReadExt};

use super::{IpVersion, Protocol, SocketInfo};

async fn parse_proc_net(
    path: &str,
    proto: Protocol,
    version: IpVersion,
) -> io::Result<HashMap<u64, (IpAddr, u16, Protocol, IpVersion)>> {
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
                let local_addr = match version {
                    IpVersion::V4 => {
                        if addr_hex.len() == 8 {
                            let ip = u32::from_str_radix(addr_hex, 16).unwrap();
                            let b1 = (ip & 0xff) as u8;
                            let b2 = ((ip >> 8) & 0xff) as u8;
                            let b3 = ((ip >> 16) & 0xff) as u8;
                            let b4 = ((ip >> 24) & 0xff) as u8;
                            IpAddr::V4(Ipv4Addr::new(b1, b2, b3, b4))
                        } else {
                            continue;
                        }
                    }
                    IpVersion::V6 => {
                        if addr_hex.len() == 32 {
                            let mut bytes = [0u8; 16];
                            for i in 0..16 {
                                let byte =
                                    u8::from_str_radix(&addr_hex[i * 2..i * 2 + 2], 16).unwrap();
                                bytes[15 - i] = byte;
                            }
                            IpAddr::V6(Ipv6Addr::from_octets(bytes))
                        } else {
                            continue;
                        }
                    }
                };

                map.insert(inode, (local_addr, port, proto.clone(), version.clone()));
            }
        }
    }
    Ok(map)
}

async fn find_process_for_inode(inode: u64) -> io::Result<Option<(u32, String)>> {
    let mut proc_iter = fs::read_dir("/proc").await?;

    while let Some(proc_entry) = proc_iter.next_entry().await? {
        let pid_str = proc_entry.file_name().into_string().unwrap_or_default();

        if let Ok(pid) = pid_str.parse::<u32>() {
            let fd_dir = format!("/proc/{}/fd", pid);
            if let Ok(mut fd_list) = fs::read_dir(&fd_dir).await {
                while let Some(fd) = fd_list.next_entry().await? {
                    if let Ok(link) = fs::read_link(fd.path()).await
                        && link
                            .to_string_lossy()
                            .contains(&format!("socket:[{}]", inode))
                    {
                        let comm_path = format!("/proc/{}/comm", pid);
                        let process_name = fs::read_to_string(comm_path)
                            .await
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        return Ok(Some((pid, process_name)));
                    }
                }
            }
        }
    }

    Ok(None)
}

pub(super) async fn get_sockets_info() -> io::Result<Vec<SocketInfo>> {
    let mut inode_map = HashMap::new();

    inode_map.extend(parse_proc_net("/proc/net/tcp", Protocol::TCP, IpVersion::V4).await?);

    inode_map.extend(parse_proc_net("/proc/net/tcp6", Protocol::TCP, IpVersion::V6).await?);

    inode_map.extend(parse_proc_net("/proc/net/udp", Protocol::UDP, IpVersion::V4).await?);

    inode_map.extend(parse_proc_net("/proc/net/udp6", Protocol::UDP, IpVersion::V6).await?);

    let mut results = Vec::new();

    for (inode, (local_addr, local_port, protocol, ip_version)) in inode_map {
        if let Ok(Some((pid, name))) = find_process_for_inode(inode).await {
            results.push(SocketInfo {
                pid,
                process_name: name,
                protocol,
                ip_version,
                local_addr,
                local_port,
            });
        }
    }

    Ok(results)
}
