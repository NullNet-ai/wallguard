use std::io;
use std::net::SocketAddr;
use tokio::process::Command;

pub(crate) async fn udp_sockets() -> io::Result<Vec<(SocketAddr, u32)>> {
    let output = Command::new("sockstat")
        .args(&["-4", "-P", "udp"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "sockstat command failed",
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sockets = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            // Format: USER COMMAND PID FD PROTO LOCAL REMOTE
            if let Ok(pid) = parts[2].parse::<u32>() {
                if let Ok(addr) = parts[5].parse::<SocketAddr>() {
                    if let SocketAddr::V4(_) = addr {
                        sockets.push((addr, pid));
                    }
                }
            }
        }
    }

    Ok(sockets)
}

pub(crate) async fn udp6_sockets() -> io::Result<Vec<(SocketAddr, u32)>> {
    let output = Command::new("sockstat")
        .args(&["-6", "-P", "udp"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "sockstat command failed",
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sockets = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            // Format: USER COMMAND PID FD PROTO LOCAL REMOTE
            if let Ok(pid) = parts[2].parse::<u32>() {
                if let Ok(addr) = parts[5].parse::<SocketAddr>() {
                    if let SocketAddr::V6(_) = addr {
                        sockets.push((addr, pid));
                    }
                }
            }
        }
    }

    Ok(sockets)
}
