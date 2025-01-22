use crate::constants::UUID;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// IP address of the gRPC server
    #[arg(short, long, default_value = "localhost")]
    pub addr: String,
    /// Port of the gRPC server
    #[arg(short, long, default_value_t = 50051)]
    pub port: u16,
    /// PCAP snaplen value (bytes)
    #[arg(short, long, default_value_t = 96)]
    pub snaplen: i32,
    /// Machine UUID
    #[arg(short, long, default_value = UUID.as_str())]
    pub uuid: String,
    /// Target platform
    #[arg(short = 't', long, default_value = "pfsense")]
    pub platform: String,

}
