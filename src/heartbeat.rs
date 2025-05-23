use crate::cli::Args;
use crate::constants::DISK_SIZE;
use crate::data_transmission::dump_dir::DumpDir;
use crate::data_transmission::grpc_handler::handle_connection_and_retransmission;
use crate::data_transmission::transmission_manager::TransmissionManager;
use crate::remote_access::RemoteAccessManager;
use futures_util::StreamExt;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{DeviceStatus, HeartbeatResponse, WallGuardGrpcInterface};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::lookup_host;
use tokio::sync::{Mutex, RwLock};

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

    // for long-running tasks (i.e., packets and system resources transmission),
    // we need to properly handle reconnections: use a separate task to check if the interface is still healthy
    let client = Arc::new(Mutex::new(None));
    let dump_bytes = (u64::from(args.disk_percentage) * *DISK_SIZE) / 100;
    log::info!("Will use at most {dump_bytes} bytes of disk for packets and resources dump files");
    let dump_dir = DumpDir::new(dump_bytes).await;
    let addr = args.addr.clone();
    let client_2 = client.clone();
    let dump_dir_2 = dump_dir.clone();
    let token_2 = token.clone();
    tokio::spawn(async move {
        handle_connection_and_retransmission(&addr, args.port, client_2, dump_dir_2, token_2).await;
    });

    let mut tx_mng = TransmissionManager::new(
        args.clone(),
        client.clone(),
        dump_dir.clone(),
        token.clone(),
    );

    loop {
        let Some(mut c) = client.lock().await.clone() else {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        };
        let Ok(mut heartbeat_stream) = c
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
            handle_hb_response(
                &heartbeat_response,
                &mut ra_mng,
                &mut tx_mng,
                client.clone(),
            )
            .await;
            let mut t = token.write().await;
            *t = heartbeat_response.token;
            drop(t);
        }
    }
}

async fn handle_hb_response(
    response: &HeartbeatResponse,
    ra_mng: &mut RemoteAccessManager,
    tx_mng: &mut TransmissionManager,
    client: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
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
            client.clone(),
            String::from("shell"),
        )
        .await
        {
            log::error!("Failed to initiate r.a. session: {err:?}");
        }
    }

    if !response.remote_ssh_enabled && ra_mng.has_ssh_session() {
        log::info!("Terminating remote access session");
        if let Err(err) = ra_mng.terminate_ssh_session().await {
            log::error!("Failed to terminate r.a. session: {err:?}");
        }
    } else if response.remote_ssh_enabled && !ra_mng.has_ssh_session() {
        log::info!("Initiating remote access session");
        if let Err(err) = establish_remote_access_session(
            response.token.clone(),
            ra_mng,
            client,
            String::from("ssh"),
        )
        .await
        {
            log::error!("Failed to initiate r.a. session: {err:?}");
        }
    }

    if !response.is_packet_capture_enabled && tx_mng.has_packet_capture() {
        tx_mng.terminate_packet_capture();
    } else if response.is_packet_capture_enabled && !tx_mng.has_packet_capture() {
        tx_mng.start_packet_capture();
    }

    if !response.is_resource_monitoring_enabled && tx_mng.has_resource_monitoring() {
        tx_mng.terminate_resource_monitoring();
    } else if response.is_resource_monitoring_enabled && !tx_mng.has_resource_monitoring() {
        tx_mng.start_resource_monitoring();
    }
}

async fn establish_remote_access_session(
    token: String,
    ra_mng: &mut RemoteAccessManager,
    client: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    session_type: String,
) -> Result<(), Error> {
    let Some(mut client) = client.lock().await.clone() else {
        return Err("Server is offline").handle_err(location!());
    };

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

        "ssh" => {
            let port = response
                .ssh_port
                .ok_or("Cannot spawn SSH remote access session, because port field is missing")
                .handle_err(location!())?;

            let key = response
                .ssh_key
                .ok_or("Cannot spawn SSH remote access session, because key field is missing")
                .handle_err(location!())?;

            ra_mng.start_ssh_session(response.id, port, &key).await
        }
        r#type => {
            Err(format!("Unsupported remote access type: {}", r#type)).handle_err(location!())
        }
    }
}
