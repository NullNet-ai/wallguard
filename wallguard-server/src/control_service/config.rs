use std::net::SocketAddr;
use std::str::FromStr;

/// Configuration for the control service, including the network address it binds to.
pub struct ControlServiceConfig {
    /// The socket address (IP + port) where the control service will listen.
    pub(crate) addr: SocketAddr,
}

impl Default for ControlServiceConfig {
    fn default() -> Self {
        let addr = SocketAddr::from_str("127.0.0.1:50051").unwrap();
        ControlServiceConfig { addr }
    }
}

impl ControlServiceConfig {
    /// Creates a new configuration by reading environment variables:
    ///
    /// - `CONTROL_SERVICE_ADDR`: The IP address to bind to (default: `127.0.0.1`).
    /// - `CONTROL_SERVICE_PORT`: The port to listen on (default: `50051`).
    ///
    /// Returns an error if either environment variable is set but contains invalid values.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if:
    /// - `CONTROL_SERVICE_ADDR` is present but cannot be parsed as a valid IP address.
    /// - `CONTROL_SERVICE_PORT` is present but cannot be parsed as a valid port number (`u16`).
    pub fn from_env() -> Self {
        let host = std::env::var("CONTROL_SERVICE_ADDR").ok();
        let port = std::env::var("CONTROL_SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok());

        if let (Some(host), Some(port)) = (host, port) {
            if let Ok(addr) = format!("{}:{}", host, port).parse::<SocketAddr>() {
                return Self { addr };
            }
        }

        Self::default()
    }
}
