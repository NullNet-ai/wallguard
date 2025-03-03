mod authentication;
mod cli;
mod config_monitor;
mod constants;
mod heartbeat;
mod packet_transmitter;
mod timer;
mod utils;

use crate::packet_transmitter::transmitter::transmit_packets;
use authentication::AuthHandler;
use clap::Parser;
use config_monitor::ConfigurationMonitor;
use libwallguard::{Authentication, DeviceStatus, SetupRequest, WallGuardGrpcInterface};

async fn setup_request(auth: &AuthHandler, args: &cli::Args) -> Result<(), String> {
    let token = auth.obtain_token_safe().await.expect("Unauthenticated");

    let _ = WallGuardGrpcInterface::new(&args.addr, args.port)
        .await
        .setup_client(SetupRequest {
            auth: Some(Authentication { token }),
            device_version: args.version.clone(),
            device_uuid: args.uuid.clone(),
        })
        .await?;

    Ok(())
}

async fn fetch_status(auth: &AuthHandler, args: &cli::Args) -> Result<DeviceStatus, String> {
    let token = auth.obtain_token_safe().await.expect("Unauthenticated");

    let response = WallGuardGrpcInterface::new(&args.addr, args.port)
        .await
        .device_status(token)
        .await?;

    let status = DeviceStatus::try_from(response.status)
        .map_err(|e| format!("Wrong DeviceStatus value: {}", e.0))?;

    Ok(status)
}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

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

    let auth = AuthHandler::new(
        args.app_id.clone(),
        args.app_secret.clone(),
        args.addr.clone(),
        args.port,
    );

    let status = fetch_status(&auth, &args)
        .await
        .expect("Failed to fetch device status");

    if status == DeviceStatus::DsDraft {
        setup_request(&auth, &args)
            .await
            .expect("Setup request failed");
    } else if status == DeviceStatus::DsArchived || status == DeviceStatus::DsDeleted {
        log::error!("Device is either archived or deleted, aborting ...",);
        return;
    }

    if cfg!(not(feature = "no-cfg-monitor")) {
        let mut cfg_monitor = ConfigurationMonitor::new(&args, auth.clone(), None)
            .await
            .expect("Failed to initialize configuration monitor");

        cfg_monitor.upload_current().await.expect(
            "Failed to capture current configuration and \\ or updaload the snapshot to the server.",
        );

        tokio::spawn(async move { cfg_monitor.watch().await });
    }

    let auth_copy = auth.clone();
    let args_copy = args.clone();
    tokio::spawn(async move { heartbeat::routine(auth_copy, args_copy).await });

    transmit_packets(args, auth.clone()).await;
}
