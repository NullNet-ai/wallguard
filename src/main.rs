mod cli;
mod confmon_handle;
mod constants;
mod packet_transmitter;

use crate::packet_transmitter::transmitter::transmit_packets;
use clap::Parser;
use wallguard_server::{Authentication, SetupRequest, WallGuardGrpcInterface};

async fn authenticate(addr: &str, port: u16, app_id: &str, app_secret: &str) -> String {
    WallGuardGrpcInterface::new(addr, port)
        .await
        .login(app_id.to_string(), app_secret.to_string())
        .await
        .expect("Authentication failed")
}

async fn setup(addr: &str, port: u16, token: &str, uuid: &str) {
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

    println!("Successful Authentication: {token:?}");

    setup(args.addr.as_str(), args.port, &token, &args.uuid).await;

    println!("Successful Setup");

    let mut cfg_watcher =
        confmon_handle::init_confmon(args.addr.clone(), args.port, &args.target).await;

    let cfg_monitoring_future = cfg_watcher.watch();

    transmit_packets(args, token).await;
    cfg_monitoring_future.await.unwrap();
}

// @TODO:
// - Implement token renewal mechanism
// - Pass token to configuration watcher's callback
