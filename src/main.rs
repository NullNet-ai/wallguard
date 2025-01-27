mod cli;
mod confmon_handle;
mod constants;
mod packet_transmitter;

use crate::packet_transmitter::transmitter::transmit_packets;
use clap::Parser;
use wallguard_server::{Authentication, SetupRequest, WallGuardGrpcInterface};

async fn authenticate(addr: &str, port: u16, app_id: &str, app_secret: &str) -> String {
    if cfg!(feature = "no-datastore") {
        println!("Datastore functionality is disabled. Using an empty token...");
        return String::new();
    }

    let token = WallGuardGrpcInterface::new(addr, port)
        .await
        .login(app_id.to_string(), app_secret.to_string())
        .await
        .expect("Authentication failed");
    println!("Successful Authentication: {token:?}");
    token
}

async fn setup(addr: &str, port: u16, token: &str, uuid: &str) {
    if cfg!(feature = "no-datastore") {
        return;
    }
    WallGuardGrpcInterface::new(addr, port)
        .await
        .setup_client(SetupRequest {
            auth: Some(Authentication {
                token: token.to_string(),
            }),
            device_version: "2.7-RELEASE".to_string(),
            device_uuid: uuid.to_string(),
            hostname: "domain.nullnet.ai".to_string(),
        })
        .await
        .expect("Setup Request Failed");
    println!("Successful Setup");
}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    println!("Arguments: {args:?}");

    let token = authenticate(
        args.addr.as_str(),
        args.port,
        args.app_id.as_str(),
        args.app_secret.as_str(),
    )
    .await;

    setup(args.addr.as_str(), args.port, &token, &args.uuid).await;

    let mut cfg_watcher =
        confmon_handle::init_confmon(args.addr.clone(), args.port, &args.target).await;

    tokio::spawn(async move {
        cfg_watcher
            .watch()
            .await
            .expect("Failed to watch configuration changes");
    });

    transmit_packets(args, token).await;
}

// @TODO:
// - Implement token renewal mechanism
// - Pass token to configuration watcher's callback
