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
use nullnet_liberror::{location, Error, ErrorHandler, Location};
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

// async fn watch_config_files(
//     interface: WGServer,
//     platform: Platform,
//     token_provider: TokenProvider,
// ) -> Result<(), Error> {
//     let files = platform.get_sysconf_files();

//     if files.is_empty() {
//         return Ok(());
//     }

//     let mut watcher = Watcher::new(files, POLL_INTERVAL).await?;
//     let detector = Detector::new(platform);

//     let mut last_state = detector.check().await;

//     // Upload current
//     upload_current_version(
//         &interface.clone(),
//         &watcher,
//         last_state,
//         token_provider.clone(),
//         platform,
//     )
//     .await?;

//     loop {
//         let mut changed = watcher.check().await.unwrap_or(false);
//         let current_state = detector.check().await;

//         if last_state == State::Draft && current_state == State::Applied {
//             changed = true;
//         }

//         last_state = current_state;

//         if changed {
//             log::info!("Config change detected, uploading new version to the server");
//             upload_current_version(
//                 &interface.clone(),
//                 &watcher,
//                 current_state,
//                 token_provider.clone(),
//                 platform,
//             )
//             .await?;
//         }

//         watcher.tick().await;
//     }
// }

// async fn upload_current_version(
//     interface: &WGServer,
//     watcher: &Watcher,
//     state: State,
//     token_provider: TokenProvider,
//     platform: Platform,
// ) -> Result<(), Error> {
//     let mut snapshot = Snapshot::new();

//     for file in &watcher.files {
//         let content = tokio::fs::read(&file.path).await.handle_err(location!())?;

//         let filename = file
//             .path
//             .file_name()
//             .unwrap_or(file.path.as_os_str())
//             .to_string_lossy()
//             .into_owned();

//         snapshot.push(FileData { filename, content });
//     }

//     let data = ConfigSnapshot {
//         configuration: Some(Fireparse::parse(snapshot, platform)?),
//         token: token_provider
//             .get()
//             .await
//             .ok_or("Failed to obtain token")
//             .handle_err(location!())?,
//         status: match state {
//             State::Draft => ConfigStatus::CsDraft.into(),
//             State::Applied => ConfigStatus::CsApplied.into(),
//             State::Undefined => ConfigStatus::CsUndefined.into(),
//         },
//     };

//     interface.handle_config_data(data).await?;

//     Ok(())
// }

// async fn watch_nftables_config(
//     interface: WGServer,
//     token_provider: TokenProvider,
// ) -> Result<(), Error> {
//     const DEFAULT_PROGRAM: Option<&str> = None;
//     const DEFAULT_PARAMETERS: &[&str] = &[];

//     let ruleset = nftables::helper::get_current_ruleset_raw(DEFAULT_PROGRAM, DEFAULT_PARAMETERS)
//         .handle_err(location!())?;

//     let mut prev_ruleset_hash = utilities::hash::sha256_digest_bytes(&ruleset);

//     let mut snapshot = Snapshot::new();
//     snapshot.push(FileData {
//         filename: "#NFRuleset".into(),
//         content: ruleset.into(),
//     });

//     let data = ConfigSnapshot {
//         configuration: Some(Fireparse::parse(snapshot, Platform::NfTables)?),
//         token: token_provider
//             .get()
//             .await
//             .ok_or("Failed to obtain token")
//             .handle_err(location!())?,
//         status: ConfigStatus::CsApplied.into(),
//     };

//     interface.handle_config_data(data).await?;

//     loop {
//         let ruleset =
//             nftables::helper::get_current_ruleset_raw(DEFAULT_PROGRAM, DEFAULT_PARAMETERS)
//                 .handle_err(location!())?;

//         let hash = utilities::hash::sha256_digest_bytes(&ruleset);

//         if prev_ruleset_hash != hash {
//             prev_ruleset_hash = hash;

//             let mut snapshot = Snapshot::new();
//             snapshot.push(FileData {
//                 filename: "#NFRuleset".into(),
//                 content: ruleset.into(),
//             });

//             let data = ConfigSnapshot {
//                 configuration: Some(Fireparse::parse(snapshot, Platform::NfTables)?),
//                 token: token_provider
//                     .get()
//                     .await
//                     .ok_or("Failed to obtain token")
//                     .handle_err(location!())?,
//                 status: ConfigStatus::CsApplied.into(),
//             };

//             interface.handle_config_data(data).await?;
//         }

//         tokio::time::sleep(POLL_INTERVAL).await;
//     }
// }
