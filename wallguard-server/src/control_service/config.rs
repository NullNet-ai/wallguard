use std::net::SocketAddr;
use std::str::FromStr;

pub struct ControlServiceConfig {
    pub(crate) addr: SocketAddr,
}

impl Default for ControlServiceConfig {
    fn default() -> Self {
        let addr = SocketAddr::from_str("127.0.0.1:50051").unwrap();
        ControlServiceConfig { addr }
    }
}

impl ControlServiceConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("CONTROL_SERVICE_ADDR").ok();
        let port = std::env::var("CONTROL_SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok());

        if let (Some(host), Some(port)) = (host, port)
            && let Ok(addr) = format!("{host}:{port}",).parse::<SocketAddr>()
        {
            return Self { addr };
        }

        Self::default()
    }
}
