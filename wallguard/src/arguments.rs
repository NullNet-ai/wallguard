use clap::Parser;

/// Application configuration from CLI arguments
#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "wallguard",
    about = "WallGuard agent that monitors the device and provides remote access."
)]
pub struct Arguments {
    /// URL of the gRPC server
    #[arg(long)]
    pub control_channel_url: String,

    /// Target platform
    #[arg(long)]
    pub platform: String,
}
