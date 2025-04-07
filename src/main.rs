mod cli;
mod config_monitor;
mod constants;
mod heartbeat;
mod packet_transmitter;
mod rtty;
mod timer;

use crate::packet_transmitter::transmitter::transmit_packets;
use clap::Parser;
use config_monitor::ConfigurationMonitor;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    let args_copy = args.clone();

    let datastore_logger_config = nullnet_liblogging::DatastoreConfig::new(
        args.app_id.clone(),
        args.app_secret.clone(),
        args.addr.clone(),
        args.port,
    );
    let logger_config =
        nullnet_liblogging::LoggerConfig::new(true, true, Some(datastore_logger_config), vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    log::info!("Arguments: {args:?}");

    let token = Arc::new(RwLock::new(String::new()));
    let token_copy = token.clone();

    tokio::spawn(async move { heartbeat::routine(token_copy, args_copy).await });
    log::info!("Waiting for the first server heartbeat");
    while token.read().await.is_empty() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    log::info!("Received the first server heartbeat");

    if cfg!(not(feature = "no-cfg-monitor")) {
        let mut cfg_monitor = ConfigurationMonitor::new(&args, token.clone(), None)
            .await
            .expect("Failed to initialize configuration monitor");

        cfg_monitor.upload_current().await.expect(
            "Failed to capture current configuration and \\ or updaload the snapshot to the server.",
        );

        tokio::spawn(async move { cfg_monitor.watch().await });
    }

    transmit_packets(args, token.clone()).await;
}
