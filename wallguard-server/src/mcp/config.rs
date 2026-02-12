use std::net::SocketAddr;

pub struct McpConfig {
    pub(crate) addr: SocketAddr,
}

impl McpConfig {
    /// Constructs a `McpConfig` from the environment variables
    /// `MCP_SERVER_HOST` and `MCP_SERVER_PORT`.
    ///
    /// Falls back to `Default` if either environment variable is missing or invalid.
    pub fn from_env() -> Self {
        let host = std::env::var("MCP_SERVER_HOST").ok();
        let port = std::env::var("MCP_SERVER_PORT")
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

impl Default for McpConfig {
    fn default() -> Self {
        let addr = "0.0.0.0:8000".parse().unwrap();
        Self { addr }
    }
}

pub const SERVICE_INSTRUCTIONS: &str = "
This server provides tools to interact with the device connected to the system.

Tools:
- execute_command: Execute a CLI command
- obtain_device_info: Obtain information about the device.
";
