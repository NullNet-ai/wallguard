use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const DEFAULT_TUNNEL_ACCEPTOT_PORT: u16 = 7777;
pub struct Config {
    pub(super) addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Self {
        let port = std::env::var("TUNNEL_ACCEPTOR_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(DEFAULT_TUNNEL_ACCEPTOT_PORT);

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

        Self { addr }
    }
}
