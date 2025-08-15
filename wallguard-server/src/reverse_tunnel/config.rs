use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ReverseTunnelConfig {
    pub(crate) addr: SocketAddr,
}

impl ReverseTunnelConfig {
    /// Constructs a `ReverseTunnelConfig` from the environment variables
    /// `REVERSE_TUNNEL_HOST` and `REVERSE_TUNNEL_PORT`.
    ///
    /// Falls back to `Default` if either environment variable is missing or invalid.
    pub fn from_env() -> Self {
        let host = std::env::var("REVERSE_TUNNEL_HOST").ok();
        let port = std::env::var("REVERSE_TUNNEL_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok());

        if let (Some(host), Some(port)) = (host, port)
            && let Ok(addr) = format!("{host}:{port}").parse::<SocketAddr>()
        {
            return Self { addr };
        }

        Self::default()
    }
}

impl Default for ReverseTunnelConfig {
    fn default() -> Self {
        let addr = "127.0.0.1:7777".parse().unwrap();
        Self { addr }
    }
}
