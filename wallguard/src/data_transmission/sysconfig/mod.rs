use crate::{
    client_data::Platform,
    data_transmission::sysconfig::{
        interfaces::InterfaceSnapshot,
        types::{FileData, Snapshot},
    },
    token_provider::TokenProvider,
    wg_server::WGServer,
};
use detector::{Detector, State};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::time::Duration;
use tokio::sync::broadcast;
use wallguard_common::protobuf::wallguard_service::{ConfigSnapshot, ConfigStatus, FileSnapshot};
use watcher::Watcher;

mod detector;
mod interfaces;
mod serde_ext;
mod types;
mod utils;
mod watcher;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub async fn watch_sysconfig(
    interface: WGServer,
    platform: Platform,
    token_provider: TokenProvider,
    mut terminate: broadcast::Receiver<()>,
) {
    tokio::select! {
        _ = terminate.recv() => {},
        _ = watch_config_files(interface, platform, token_provider) => {}
    }
}

async fn watch_config_files(
    interface: WGServer,
    platform: Platform,
    token_provider: TokenProvider,
) -> Result<(), Error> {
    let files = platform.get_sysconf_files();

    if files.is_empty() {
        return Ok(());
    }

    let mut watcher = Watcher::new(files, POLL_INTERVAL).await?;
    let detector = Detector::new(platform);

    let mut last_state = detector.check().await;

    // Upload current
    upload_current_version(
        &interface.clone(),
        &watcher,
        last_state,
        token_provider.clone(),
    )
    .await?;

    loop {
        let mut changed = watcher.check().await.unwrap_or(false);
        let current_state = detector.check().await;

        if last_state == State::Draft && current_state == State::Applied {
            changed = true;
        }

        last_state = current_state;

        if changed {
            log::info!("Config change detected, uploading new version to the server");
            upload_current_version(
                &interface.clone(),
                &watcher,
                current_state,
                token_provider.clone(),
            )
            .await?;
        }

        watcher.tick().await;
    }

    async fn upload_current_version(
        interface: &WGServer,
        watcher: &Watcher,
        state: State,
        token_provider: TokenProvider,
    ) -> Result<(), Error> {
        let mut snapshot = Snapshot::new();

        for file in &watcher.files {
            let content = tokio::fs::read(&file.path).await.handle_err(location!())?;

            let filename = file
                .path
                .file_name()
                .unwrap_or(file.path.as_os_str())
                .to_string_lossy()
                .into_owned();

            snapshot.push(FileData { filename, content });
        }

        let ifaces_data = InterfaceSnapshot::take_all();
        let blob = InterfaceSnapshot::serialize_snapshot(&ifaces_data).handle_err(location!())?;

        snapshot.push(FileData {
            filename: "#NetworkInterfaces".to_string(),
            content: blob,
        });

        let data = ConfigSnapshot {
            files: snapshot
                .iter()
                .map(|fs| FileSnapshot {
                    filename: fs.filename.clone(),
                    contents: fs.content.clone(),
                })
                .collect(),
            token: token_provider
                .get()
                .await
                .ok_or("Failed to obtain token")
                .handle_err(location!())?,
            status: match state {
                State::Draft => ConfigStatus::CsDraft.into(),
                State::Applied => ConfigStatus::CsApplied.into(),
                State::Undefined => ConfigStatus::CsUndefined.into(),
            },
        };

        interface.handle_config_data(data).await?;

        Ok(())
    }
}
