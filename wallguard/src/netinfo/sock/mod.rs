use std::net::SocketAddr;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "freebsd")]
mod freebsd;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    TCP,
    UDP,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpVersion {
    V4,
    V6,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketInfo {
    pub process_name: String,
    pub protocol: Protocol,
    pub sockaddr: SocketAddr,
}

#[cfg(target_os = "linux")]
async fn get_sockets_info_impl() -> Vec<SocketInfo> {
    linux::get_sockets_info().await
}

#[cfg(target_os = "windows")]
async fn get_sockets_info_impl() -> Vec<SocketInfo> {
    windows::get_sockets_info()
}

#[cfg(target_os = "freebsd")]
async fn get_sockets_info_impl() -> Vec<SocketInfo> {
    freebsd::get_sockets_info().await
}

pub async fn get_sockets_info() -> Vec<SocketInfo> {
    get_sockets_info_impl().await
}
