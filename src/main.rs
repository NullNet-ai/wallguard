mod authentication;
mod cli;
mod config_monitor;
mod constants;
mod heartbeat;
mod packet_transmitter;
mod utils;

use crate::packet_transmitter::transmitter::transmit_packets;
use authentication::AuthHandler;
use clap::Parser;
use config_monitor::ConfigurationMonitor;
use libwallguard::{Authentication, SetupRequest, WallGuardGrpcInterface};

async fn setup_request(auth: &AuthHandler, args: &cli::Args) -> Result<(), String> {
    let token = auth.obtain_token_safe().await.expect("Unauthenticated");

    let response = WallGuardGrpcInterface::new(&args.addr, args.port)
        .await
        .setup_client(SetupRequest {
            auth: Some(Authentication { token }),
            device_version: args.version.clone(),
            device_uuid: args.uuid.clone(),
            hostname: args.hostname.clone(),
        })
        .await?;

    if response.success {
        Ok(())
    } else {
        Err(response.message)
    }
}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    println!("Arguments: {args:?}");

    let auth = AuthHandler::new(
        args.app_id.clone(),
        args.app_secret.clone(),
        args.addr.clone(),
        args.port,
    );

    let mut cfg_monitor = ConfigurationMonitor::new(&args, auth.clone(), None)
        .await
        .expect("Failed to initialize configuration monitor");

    cfg_monitor.upload_current().await.expect(
        "Failed to capture current configuration and \\ or updaload the snapshot to the server.",
    );

    setup_request(&auth, &args)
        .await
        .expect("Setup request failed");

    tokio::spawn(async move { cfg_monitor.watch().await });

    let auth_copy = auth.clone();
    let args_copy = args.clone();
    tokio::spawn(async move { heartbeat::routine(auth_copy, args_copy).await });

    transmit_packets(args, auth.clone()).await;
}
