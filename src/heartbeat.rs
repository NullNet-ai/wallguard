use std::time::Duration;

use libwallguard::{DeviceStatus, HeartbeatResponse, WallGuardGrpcInterface};
use log::Level;

use crate::authentication::AuthHandler;
use crate::cli::Args;
use crate::logger::Logger;

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
                    Err(msg) => Logger::log(
                        Level::Error,
                        format!("Heartbeat: Request failed failed - {msg}"),
                    ),
                }
            }
            Err(msg) => Logger::log(
                Level::Error,
                format!("Heartbeat: Authentication failed - {msg}"),
            ),
        };

        tokio::time::sleep(interval).await;
    }
}

fn handle_hb_response(response: HeartbeatResponse) {
    match DeviceStatus::try_from(response.status) {
        Ok(DeviceStatus::DsArchived | DeviceStatus::DsDeleted) => {
            Logger::log(
                Level::Warn,
                "Device has been archived or deleted, aborting execution ...",
            );
            std::process::exit(0);
        }
        Ok(_) => {}
        Err(_) => Logger::log(
            Level::Error,
            format!("Unknown device status value {}", response.status),
        ),
    }
}
