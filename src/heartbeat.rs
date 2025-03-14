use std::time::Duration;

use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{DeviceStatus, HeartbeatResponse, WallGuardGrpcInterface};

use crate::authentication::AuthHandler;
use crate::cli::Args;
use crate::remote_access::RemoteAccessManager;

// @TODO
// Refactor this file.

fn create_remote_access_manager(args: &Args) -> RemoteAccessManager {
    let platform =
        nullnet_libconfmon::Platform::from_string(&args.target).expect("Unsupported platform");

    let server_addr = format!("{}:{}", args.addr, args.port).parse().expect("Failed to parse server addr");
    RemoteAccessManager::new(platform, server_addr)
}

pub async fn routine(auth: AuthHandler, args: Args) {
    let interval = Duration::from_secs(args.heartbeat_interval);

    let mut ra_mng = create_remote_access_manager(&args);

    loop {
        match auth.obtain_token_safe().await {
            Ok(token) => {
                let mut client = WallGuardGrpcInterface::new(&args.addr, args.port).await;

                match client.heartbeat(token.clone()).await {
                    Ok(response) => {
                        handle_hb_response(response, token, &mut ra_mng, client).await;
                    }
                    Err(msg) => log::error!("Heartbeat: Request failed failed - {msg}"),
                }
            }
            Err(msg) => log::error!("Heartbeat: Authentication failed - {msg}"),
        };

        tokio::time::sleep(interval).await;
    }
}

async fn handle_hb_response(
    response: HeartbeatResponse,
    token: String,
    ra_mng: &mut RemoteAccessManager,
    client: WallGuardGrpcInterface,
) {
    match DeviceStatus::try_from(response.status) {
        Ok(DeviceStatus::DsArchived | DeviceStatus::DsDeleted) => {
            log::warn!("Device has been archived or deleted, aborting execution ...",);
            std::process::exit(0);
        }
        Ok(_) => {}
        Err(_) => log::error!("Unknown device status value {}", response.status),
    };

    if !response.is_remote_access_enabled && ra_mng.has_session() {
        if let Err(err) = ra_mng.terminate().await {
            log::error!("Failed to terminate r.a. session: {err:?}");
        }
    } else if response.is_remote_access_enabled && !ra_mng.has_session() {
        if let Err(err) = establish_remote_access_session(token, ra_mng, client).await {
            log::error!("Failed to terminate r.a. session: {err:?}");
        }
    }
}

async fn establish_remote_access_session(
    token: String,
    ra_mng: &mut RemoteAccessManager,
    mut client: WallGuardGrpcInterface,
) -> Result<(), Error> {
    let response = client
        .request_control_channel(token)
        .await
        .handle_err(location!())?;

    match response.r#type.to_lowercase().as_str() {
        "shell" => ra_mng.start_tty_session(response.id).await,
        "ui" => {
            let protocol = response
                .protocol
                .ok_or("Cannot spawn UI remote access session, because ptorocol field is missing")
                .handle_err(location!())?;
            ra_mng.start_ui_session(response.id, &protocol).await
        }
        r#type => {
            Err(format!("Unsupported remote access type: {}", r#type)).handle_err(location!())
        }
    }
}
