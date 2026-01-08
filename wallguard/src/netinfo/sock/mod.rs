use std::net::IpAddr;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "freebsd")]
mod freebsd;

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
    Dual
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketInfo {
    pub pid: u32,
    pub process_name: String,
    pub protocol: Protocol,
    pub ip_version: IpVersion,
    pub local_addr: IpAddr,
    pub local_port: u16,
}

pub async fn get_sockets_info() -> Vec<SocketInfo> {
    #[cfg(target_os = "linux")]
    {
        return linux::get_sockets_info().await.unwrap_or_default();
    }

    #[cfg(target_os = "freebsd")]
    {
        return freebsd::get_sockets_info().await.unwrap_or_default();
    }
    
    vec![]
}
