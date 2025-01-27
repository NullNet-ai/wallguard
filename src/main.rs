mod authentication;
mod cli;
mod confmon_handle;
mod constants;
mod packet_transmitter;
mod utils;

use crate::packet_transmitter::transmitter::transmit_packets;
use authentication::AutoAuth;
use clap::Parser;
use wallguard_server::{Authentication, SetupRequest, WallGuardGrpcInterface};

async fn setup(addr: &str, port: u16, token: &str, uuid: &str) {
    if cfg!(feature = "no-datastore") {
        return;
    }

    let response = WallGuardGrpcInterface::new(addr, port)
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
// - Pass token to configuration watcher's callback
// - Implement heartbear
