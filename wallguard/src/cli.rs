use clap::Parser;

/// Application configuration from CLI arguments
#[derive(Parser, Debug, Clone)]
#[command(
    name = "wallguard",
    about = "WallGuard agent that monitors the device and provides remote access."
)]
pub struct Args {
    /// IP address of the gRPC server
    #[arg(short, long, default_value = "127.0.0.1")]
    pub addr: String,

    /// Port of the gRPC server
    #[arg(short, long, default_value_t = 50051)]
    pub port: u16,

    /// App ID
    #[arg(long)]
    pub app_id: String,

    /// App secret
    #[arg(long)]
    pub app_secret: String,

    /// IP address of the tunnel server
    #[arg(long, default_value = "127.0.0.1")]
    pub tunnel_addr: String,

    /// Port of the tunnel server
    #[arg(long, default_value_t = 7777)]
    pub tunnel_port: u16,
}

impl Args {
    /// Validate required arguments
    pub fn validate(&self) -> Result<(), String> {
        if self.app_id.trim().is_empty() {
            return Err("App ID is missing".into());
        }
        if self.app_secret.trim().is_empty() {
            return Err("App secret is missing".into());
        }
        Ok(())
    }
}
