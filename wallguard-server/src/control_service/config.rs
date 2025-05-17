use std::env;
use std::net::{IpAddr, SocketAddr};
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
    pub fn from_env() -> Result<Self, String> {
        let addr_str = env::var("CONTROL_SERVICE_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port_str = env::var("CONTROL_SERVICE_PORT").unwrap_or_else(|_| "50051".to_string());

        let ip = addr_str
            .parse::<IpAddr>()
            .map_err(|e| format!("Failed to parse CONTROL_SERVICE_ADDR: {}", e))?;

        let port = port_str
            .parse::<u16>()
            .map_err(|e| format!("Failed to parse CONTROL_SERVICE_PORT: {}", e))?;

        Ok(ControlServiceConfig {
            addr: SocketAddr::new(ip, port),
        })
    }
}
