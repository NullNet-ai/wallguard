use std::time::Duration;

use libwallguard::{DeviceStatus, HeartbeatResponse, WallGuardGrpcInterface};

use crate::authentication::AuthHandler;
use crate::cli::Args;

pub async fn routine(auth: AuthHandler, args: Args) {
    let interval = Duration::from_secs(args.heartbeat_interval);
    loop {
        match auth.obtain_token_safe().await {
            Ok(token) => {
                let mut client = WallGuardGrpcInterface::new(&args.addr, args.port).await;

                match client.heartbeat(token).await {
                    Ok(response) => {
                        handle_hb_response(response);
                    }
                    Err(msg) => log::error!("Heartbeat: Request failed failed - {msg}"),
                }
            }
            Err(msg) => log::error!("Heartbeat: Authentication failed - {msg}"),
        };

        tokio::time::sleep(interval).await;
    }
}

fn handle_hb_response(response: HeartbeatResponse) {
    match DeviceStatus::try_from(response.status) {
        Ok(DeviceStatus::DsArchived | DeviceStatus::DsDeleted) => {
            log::warn!("Device has been archived or deleted, aborting execution ...",);
            std::process::exit(0);
        }
        Ok(_) => {}
        Err(_) => log::error!("Unknown device status value {}", response.status),
    }
}
