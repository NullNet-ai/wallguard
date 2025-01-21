mod cli;
mod constants;
mod packet_transmitter;

use crate::packet_transmitter::transmitter::transmit_packets;
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    println!("Arguments: {args:?}");

    let monitor_config = traffic_monitor::MonitorConfig {
        addr: args.addr,
        snaplen: args.snaplen,
    };
    let rx = traffic_monitor::monitor_devices(&monitor_config);

    transmit_packets(&rx, monitor_config.addr, args.port, args.uuid).await;
}
