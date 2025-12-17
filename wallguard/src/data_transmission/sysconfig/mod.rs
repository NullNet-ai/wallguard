use crate::{
    client_data::Platform,
    data_transmission::sysconfig::{
        data::{FileToMonitor, SystemConfigurationFile},
        types::Snapshot,
    },
    fireparse::Fireparse,
    token_provider::TokenProvider,
    wg_server::WGServer,
};
use detector::{Detector, State};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::broadcast;
use wallguard_common::protobuf::wallguard_service::ConfigSnapshot;

pub mod data;
pub mod types;

mod detector;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub async fn watch_sysconfig(
    interface: WGServer,
    platform: Platform,
    token_provider: TokenProvider,
    mut terminate: broadcast::Receiver<()>,
) {
    tokio::select! {
        _ = terminate.recv() => {},
        _ = watch_configuration_files(interface, platform, token_provider) => {}
    }
}

pub async fn force_upload_once(
    interface: WGServer,
    platform: Platform,
    token_provider: TokenProvider,
) -> Result<(), Error> {
    let mut files = platform.get_sysconf_files();

    for file in files.iter_mut() {
        let _ = file.update().await;
    }

    if !files.is_empty() {
        let detector = Detector::new(platform);
        let state = detector.check().await;

        upload_all(
            interface.clone(),
            platform,
            state,
            token_provider.clone(),
            &mut files,
        )
        .await?;
    }

    Ok(())
}

async fn watch_configuration_files(
    interface: WGServer,
    platform: Platform,
    token_provider: TokenProvider,
) -> Result<(), Error> {
    let mut files = platform.get_sysconf_files();
    let detector = Detector::new(platform);

    let mut last_state = detector.check().await;
    let _ = update_all(&mut files).await;

    upload_all(
        interface.clone(),
        platform,
        last_state,
        token_provider.clone(),
        &mut files,
    )
    .await?;

    if files.is_empty() {
        return Ok(());
    }

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let mut changed = update_all(&mut files).await?;
        let current_state = detector.check().await;

        if last_state == State::Draft && current_state == State::Applied {
            changed = true;
        }

        last_state = current_state;

        if changed {
            log::info!("Config change detected, uploading new version to the server");
            upload_all(
                interface.clone(),
                platform,
                last_state,
                token_provider.clone(),
                &mut files,
            )
            .await?;
        }
    }
}

async fn upload_all(
    interface: WGServer,
    platform: Platform,
    state: State,
    token_provider: TokenProvider,
    files: &mut [SystemConfigurationFile],
) -> Result<(), Error> {
    let mut snapshot = Snapshot::new();

    for file in files.iter() {
        snapshot.push(file.take_snapshot());
    }

    let data = ConfigSnapshot {
        configuration: Some(Fireparse::parse(snapshot, platform)?),
        token: token_provider
            .get()
            .await
            .ok_or("Failed to obtain token")
            .handle_err(location!())?,
        status: state.into(),
    };

    interface.handle_config_data(data).await?;

    Ok(())
}

async fn update_all(files: &mut [SystemConfigurationFile]) -> Result<bool, Error> {
    let mut retval = false;

    for file in files.iter_mut() {
        retval = retval || file.update().await?;
    }

    Ok(retval)
}
