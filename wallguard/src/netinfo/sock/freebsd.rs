use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::Command;

use tokio::fs;
use tokio::io;

use super::{IpVersion, Protocol, SocketInfo};

async fn find_process_for_socket(pid: u32) -> io::Result<Option<(u32, String)>> {
    let ps_output = tokio::process::Command::new("ps")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg("comm=")
        .output()
        .await?;

    if ps_output.status.success() {
        let process_name = String::from_utf8_lossy(&ps_output.stdout)
            .trim()
            .to_string();
        if !process_name.is_empty() {
            return Ok(Some((pid, process_name)));
        }
    }

    Ok(None)
}

async fn enrich_with_process_info(
    results: Vec<(IpAddr, u16, Protocol, IpVersion, u32)>,
) -> io::Result<Vec<SocketInfo>> {
    let mut socket_infos = Vec::new();

    for (local_addr, local_port, protocol, ip_version, pid) in results {
        if let Ok(Some((_, process_name))) = find_process_for_socket(pid).await {
            socket_infos.push(SocketInfo {
                pid,
                process_name,
                protocol,
                ip_version,
                local_addr,
                local_port,
            });
        }
    }

    Ok(socket_infos)
}

fn parse_sockstat_output_with_pid(
    output: &str,
    proto: Protocol,
    version: IpVersion,
) -> io::Result<Vec<(IpAddr, u16, Protocol, IpVersion, u32)>> {
    let mut sockets = Vec::new();

    for line in output.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }

        // sockstat output: USER COMMAND PID FD PROTO LOCAL FOREIGN
        let pid_str = cols[2];
        let local_addr_port = cols[5];

        if let Ok(pid) = pid_str.parse::<u32>() {
            if let Some(colon) = local_addr_port.rfind(':') {
                let addr_str = &local_addr_port[..colon];
                let port_str = &local_addr_port[colon + 1..];

                if let Ok(port) = port_str.parse::<u16>() {
                    let local_addr = if addr_str == "*" {
                        match version {
                            IpVersion::V4 => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                            _ => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                        }
                    } else {
                        addr_str
                            .parse::<IpAddr>()
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    };

                    sockets.push((local_addr, port, proto.clone(), version.clone(), pid));
                }
            }
        }
    }

    Ok(sockets)
}

pub(super) async fn get_sockets_info() -> io::Result<Vec<SocketInfo>> {
    let mut all_sockets: Vec<(IpAddr, u16, Protocol, IpVersion, u32)> = Vec::new();

    for (proto, version) in [
        (Protocol::TCP, IpVersion::V4),
        (Protocol::TCP, IpVersion::V6),
        (Protocol::TCP, IpVersion::Dual),
        (Protocol::UDP, IpVersion::V4),
        (Protocol::UDP, IpVersion::V6),
        (Protocol::UDP, IpVersion::Dual),
    ] {
        let mut args: Vec<String> = Vec::new();

        match version {
            IpVersion::V4 => args.push("-4".into()),
            IpVersion::V6 => args.push("-6".into()),
            IpVersion::Dual => {}
        };

        args.push("-l".into());

        let proto_str = match (version, proto) {
            (IpVersion::Dual, Protocol::TCP) => "tcp46",
            (IpVersion::Dual, Protocol::UDP) => "udp46",
            (_, Protocol::TCP) => "tcp",
            (_, Protocol::UDP) => "udp",
        };

        args.push("-P".into());
        args.push(proto_str.into());

        let sockstat_output = Command::new("sockstat").args(&args).output().await?;

        if !sockstat_output.status.success() {
            continue;
        }

        let output = String::from_utf8_lossy(&sockstat_output.stdout);
        let sockets = parse_sockstat_output_with_pid(&output, proto, version)?;
        all_sockets.extend(sockets);
    }

    enrich_with_process_info(all_sockets).await
}
