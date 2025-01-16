mod cli;
mod constants;
mod net_capture;
mod proto;

use crate::net_capture::sniffer::sniff_devices;
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    println!("Arguments: {args:?}");

    sniff_devices(args).await;
}
