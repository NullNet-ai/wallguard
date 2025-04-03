use crate::cli::Args;
use futures_util::StreamExt;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{DeviceStatus, HeartbeatResponse, WallGuardGrpcInterface};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub async fn routine(token: Arc<RwLock<String>>, args: Args) -> Result<(), Error> {
    loop {
        let mut heartbeat_stream = WallGuardGrpcInterface::new(&args.addr, args.port)
            .await
            .heartbeat(
                args.app_id.clone(),
                args.app_secret.clone(),
                args.version.clone(),
                args.uuid.clone(),
            )
            .await
            .handle_err(location!())?;

        while let Some(Ok(heartbeat_response)) = heartbeat_stream.next().await {
            handle_hb_response(&heartbeat_response);
            let mut t = token.write().await;
            // todo: remove unwrap
            *t = heartbeat_response.token.clone();
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

fn handle_hb_response(response: &HeartbeatResponse) {
    match DeviceStatus::try_from(response.status) {
        Ok(DeviceStatus::DsArchived | DeviceStatus::DsDeleted) => {
            log::warn!("Device has been archived or deleted, aborting execution ...",);
            std::process::exit(0);
        }
        Ok(_) => {}
        Err(_) => log::error!("Unknown device status value {}", response.status),
    }
}
