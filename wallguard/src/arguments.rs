use clap::Parser;

/// Application configuration from CLI arguments
#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "wallguard",
    about = "WallGuard agent that monitors the device and provides remote access."
)]
pub struct Arguments {
    /// IP address of the gRPC server
    #[arg(long)]
    pub control_channel_host: String,

    /// Port of the gRPC server
    #[arg(long)]
    pub control_channel_port: u16,

    /// IP address of the tunnel server
    #[arg(long)]
    pub tunnel_host: String,

    /// Port of the tunnel server
    #[arg(long)]
    pub tunnel_port: u16,

    /// Target platform
    #[arg(long)]
    pub platform: String,
}
