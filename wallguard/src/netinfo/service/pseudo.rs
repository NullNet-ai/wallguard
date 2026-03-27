use crate::netinfo::service::ServiceInfo;
use crate::netinfo::sock::SocketInfo;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::process::Command;

async fn has_desktop() -> bool {
    if cfg!(target_os = "freebsd") {
        return false;
    }

    if env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok() {
        return true;
    }

    let output = Command::new("pgrep")
        .args(&["-x", "Xorg|Xwayland|sway|weston|mutter|kwin_wayland"])
        .output()
        .await;

    matches!(output, Ok(o) if o.status.success())
}

pub async fn filter(_: &mut Vec<SocketInfo>) -> Vec<ServiceInfo> {
    let mut retval = vec![ServiceInfo {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        protocol: super::Protocol::Tty,
        program: String::from("/wallguard-tty"),
    }];

    if has_desktop().await {
        retval.push(ServiceInfo {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            protocol: super::Protocol::Rd,
            program: String::from("/wallguard-remote-desktop"),
        });
    }

    retval
}
