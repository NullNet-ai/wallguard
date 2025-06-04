use clap::Parser;

/// Application configuration from CLI arguments
#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "wallguard",
    about = "WallGuard agent that monitors the device and provides remote access."
)]
pub struct Arguments {
    /// IP address of the gRPC server
    #[arg(short, long, default_value = "127.0.0.1")]
    pub addr: String,

    /// Port of the gRPC server
    #[arg(short, long, default_value_t = 50051)]
    pub port: u16,

    /// IP address of the tunnel server
    #[arg(long, default_value = "127.0.0.1")]
    pub tunnel_addr: String,

    /// Port of the tunnel server
    #[arg(long, default_value_t = 7777)]
    pub tunnel_port: u16,
}
