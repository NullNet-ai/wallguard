mod cli;
mod config_monitor;
mod constants;
mod heartbeat;
mod packet_transmitter;
mod remote_access;
mod resources_monitor;
mod rtty;
mod timer;

use crate::constants::DISK_SIZE;
use crate::packet_transmitter::grpc_handler::handle_connection_and_retransmission;
use crate::packet_transmitter::transmitter::transmit_packets;
use crate::resources_monitor::monitor_system_resources;
use clap::Parser;
use config_monitor::ConfigurationMonitor;
use nullnet_liblogging::ServerKind;
use remote_access::remove_added_ssh_keys;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{Mutex, RwLock};

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    let args_copy = args.clone();
    let token = Arc::new(RwLock::new(String::new()));
    let token_copy = token.clone();

    let datastore_logger_config = nullnet_liblogging::DatastoreConfig::new(
        token.clone(),
        ServerKind::WallGuard,
        args.addr.clone(),
        args.port,
        false,
    );
    let logger_config =
        nullnet_liblogging::LoggerConfig::new(true, true, Some(datastore_logger_config), vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    log::info!("Arguments: {args:?}");

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

    // for long-running tasks (i.e., packets and system resources transmission),
    // we need to properly handle reconnections: use a separate task to check if the interface is still healthy
    let client = Arc::new(Mutex::new(None));
    let client_2 = client.clone();
    let dump_bytes = (u64::from(args.disk_percentage) * *DISK_SIZE) / 100;
    log::info!("Will use at most {dump_bytes} bytes of disk space for packet dump files");
    let dump_dir = packet_transmitter::dump_dir::DumpDir::new(dump_bytes).await;
    let dump_dir_2 = dump_dir.clone();
    let token_2 = token.clone();
    let addr = args.addr.clone();
    tokio::spawn(async move {
        handle_connection_and_retransmission(&addr, args.port, client_2, dump_dir_2, token_2).await;
    });

    let mut terminate_signal = signal(SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = terminate_signal.recv() => {},
        () = transmit_packets(args, token.clone(), dump_dir, client.clone()) => {},
        () = monitor_system_resources(token.clone(), client) => {}
    }

    let _ = remove_added_ssh_keys();
}
