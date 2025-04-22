use crate::cli::Args;
use crate::remote_access::RemoteAccessManager;
use futures_util::StreamExt;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{DeviceStatus, HeartbeatResponse, WallGuardGrpcInterface};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::lookup_host;
use tokio::sync::RwLock;

async fn create_remote_access_manager(args: &Args) -> RemoteAccessManager {
    let platform =
        nullnet_libconfmon::Platform::from_string(&args.target).expect("Unsupported platform");

    let addr = format!("{}:{}", args.tunnel_addr, args.tunnel_port);
    let mut addrs = lookup_host(addr)
        .await
        .handle_err(location!())
        .expect("Failed to resolve server address");

    let server_addr = addrs
        .next()
        .ok_or("No address found")
        .handle_err(location!())
        .expect("No addresses found for server");

    RemoteAccessManager::new(platform, server_addr)
}

pub async fn routine(token: Arc<RwLock<String>>, args: Args) {
    let mut ra_mng = create_remote_access_manager(&args).await;
    loop {
        let mut client = WallGuardGrpcInterface::new(&args.addr, args.port).await;
        let Ok(mut heartbeat_stream) = client
            .heartbeat(
                args.app_id.clone(),
                args.app_secret.clone(),
                args.version.clone(),
                args.uuid.clone(),
            )
            .await
        else {
            log::warn!("Failed to send heartbeat to the server. Retrying in 10 seconds...");
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        };

        while let Some(Ok(heartbeat_response)) = heartbeat_stream.next().await {
            handle_hb_response(&heartbeat_response, &mut ra_mng, client.clone()).await;
            let mut t = token.write().await;
            *t = heartbeat_response.token;
            drop(t);
        }
    }
}

async fn handle_hb_response(
    response: &HeartbeatResponse,
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
    }

    if !response.remote_ui_enabled && ra_mng.has_ui_session() {
        log::info!("Terminating remote access session");
        if let Err(err) = ra_mng.terminate_ui_session().await {
            log::error!("Failed to terminate r.a. session: {err:?}");
        }
    } else if response.remote_ui_enabled && !ra_mng.has_ui_session() {
        log::info!("Initiating remote access session");
        if let Err(err) = establish_remote_access_session(
            response.token.clone(),
            ra_mng,
            client.clone(),
            String::from("ui"),
        )
        .await
        {
            log::error!("Failed to initiate r.a. session: {err:?}");
        }
    }

    if !response.remote_shell_enabled && ra_mng.has_shell_session() {
        log::info!("Terminating remote access session");
        if let Err(err) = ra_mng.terminate_shell_session().await {
            log::error!("Failed to terminate r.a. session: {err:?}");
        }
    } else if response.remote_shell_enabled && !ra_mng.has_shell_session() {
        log::info!("Initiating remote access session");
        if let Err(err) = establish_remote_access_session(
            response.token.clone(),
            ra_mng,
            client,
            String::from("shell"),
        )
        .await
        {
            log::error!("Failed to initiate r.a. session: {err:?}");
        }
    }
}

async fn establish_remote_access_session(
    token: String,
    ra_mng: &mut RemoteAccessManager,
    mut client: WallGuardGrpcInterface,
    session_type: String,
) -> Result<(), Error> {
    let response = client
        .request_control_channel(token, session_type)
        .await
        .handle_err(location!())?;

    match response.r#type.to_lowercase().as_str() {
        "shell" => ra_mng.start_tty_session(response.id).await,
        "ui" => {
            let protocol = response
                .protocol
                .ok_or("Cannot spawn UI remote access session, because protocol field is missing")
                .handle_err(location!())?;

            ra_mng.start_ui_session(response.id, &protocol).await
        }
        r#type => {
            Err(format!("Unsupported remote access type: {}", r#type)).handle_err(location!())
        }
    }
}
