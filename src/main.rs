mod cli;
mod confmon_handle;
mod constants;
mod packet_transmitter;

use crate::packet_transmitter::transmitter::transmit_packets;
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    println!("Arguments: {args:?}");

    // let mut cfg_watcher = confmon_handle::init_confmon(
    //     args.addr.clone(),
    //     args.port,
    //     args.uuid.clone(),
    //     args.platform.clone(),
    // ).await;
    //
    // // TODO: Take current snapshot and send to the server
    // let cfg_monitoring_future = cfg_watcher.watch();

    let monitor_config = traffic_monitor::MonitorConfig {
        addr: args.addr,
        snaplen: args.snaplen,
    };
    let rx = traffic_monitor::monitor_devices(&monitor_config);

    transmit_packets(&rx, monitor_config.addr, args.port, args.uuid).await;
    // cfg_monitoring_future.await.unwrap();
}
