use crate::constants::UUID;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// IP address of the gRPC server
    #[arg(short, long, default_value = "localhost")]
    pub addr: String,
    /// Percentage of available disk space to use for packet dump files in case of server unavailability
    #[arg(short, long, default_value_t = 50, value_parser = clap::value_parser!(u8).range(1..100))]
    pub disk_percentage: u8,
    /// Port of the gRPC server
    #[arg(short, long, default_value_t = 50051)]
    pub port: u16,
    /// PCAP snaplen value (bytes)
    #[arg(short, long, default_value_t = 96)]
    pub snaplen: i32,
    /// Target platform
    #[arg(short, long, default_value = "pfsense")]
    pub target: String,
    /// Machine UUID
    #[arg(short, long, default_value = UUID.as_str())]
    pub uuid: String,
}
