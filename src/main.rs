mod authentication;
mod cli;
mod confmon_handle;
mod constants;
mod heartbeat;
mod packet_transmitter;
mod utils;

use crate::packet_transmitter::transmitter::transmit_packets;
use authentication::AutoAuth;
use clap::Parser;
use libwallguard::{Authentication, SetupRequest, WallGuardGrpcInterface};

async fn setup(auth: &AutoAuth, args: &cli::Args) {
    if cfg!(feature = "no-datastore") {
        return;
    }

    let token = auth.obtain_token_safe().await.expect("Unauthenticated");

    let response = WallGuardGrpcInterface::new(&args.addr, args.port)
        .await
        .setup_client(SetupRequest {
            auth: Some(Authentication { token }),
            device_version: args.version.clone(),
            device_uuid: args.uuid.clone(),
            hostname: args.hostname.clone(),
        })
        .await
        .expect("Setup Request Failed");

    if response.success {
        println!("Successful Setup");
    } else {
        panic!("Setup failed: {}", response.message);
    }
}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    println!("Arguments: {args:?}");

    let auth = AutoAuth::new(
        args.app_id.clone(),
        args.app_secret.clone(),
        args.addr.clone(),
        args.port,
    );

    let token = auth
        .obtain_token_safe()
        .await
        .expect("Server authentication failed");

    setup(&auth, &args).await;

    let mut cfg_watcher =
        confmon_handle::init_confmon(args.addr.clone(), args.port, &args.target).await;

    tokio::spawn(async move {
        cfg_watcher
            .watch()
            .await
            .expect("Failed to watch configuration changes");
    });

    let auth_copy = auth.clone();
    let args_copy = args.clone();
    tokio::spawn(async move { heartbeat::routine(auth_copy, args_copy).await });

    transmit_packets(args, token).await;
}

// @TODO:
// - Pass token to configuration watcher's callback
