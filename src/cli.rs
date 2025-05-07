use crate::constants::UUID;
use clap::Parser;

const APP_ID: Option<&str> = option_env!("APP_ID");
const APP_SECRET: Option<&str> = option_env!("APP_SECRET");

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// IP address of the gRPC server
    #[arg(short, long, default_value = "127.0.0.1")]
    pub addr: String,
    /// Port of the gRPC server
    #[arg(short, long, default_value_t = 50051)]
    pub port: u16,
    /// App ID
    #[arg(long = "app_id", default_value = APP_ID.unwrap_or_default())]
    pub app_id: String,
    /// App secret
    #[arg(long = "app_secret", default_value = APP_SECRET.unwrap_or_default())]
    pub app_secret: String,
    /// Percentage of available disk space to use for packet dump files in case of server unavailability
    #[arg(short, long, default_value_t = 50, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub disk_percentage: u8,
    /// IP address of the tunnel server
    #[arg(long, default_value = "127.0.0.1")]
    pub tunnel_addr: String,
    /// Port of the tunnel server
    #[arg(long, default_value_t = 9000)]
    pub tunnel_port: u16,
    /// PCAP snaplen value (bytes)
    #[arg(short, long, default_value_t = 96)]
    pub snaplen: i32,
    /// Target platform
    #[arg(short, long, default_value = "pfsense")]
    pub target: String,
    /// Transmit interval (seconds)
    #[arg(long, default_value_t = 1)]
    pub transmit_interval: u64,
    /// Machine UUID
    #[arg(short, long, default_value = UUID.as_str())]
    pub uuid: String,
    /// Platform version
    #[arg(long, default_value = "unknown")]
    pub version: String,
}
