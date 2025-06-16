use crate::platform::Platform;
use detector::{Detector, State};
use watcher::Watcher;

use nullnet_liberror::Error;
use std::time::Duration;
use tokio::sync::broadcast;

mod detector;
mod types;
mod utils;
mod watcher;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub async fn watch_sysconfig(platform: Platform, mut terminate: broadcast::Receiver<()>) {
    tokio::select! {
        _ = terminate.recv() => {},
        _ = watch_config_files(platform) => {}
    }
}

async fn watch_config_files(platform: Platform) -> Result<(), Error> {
    let files = platform.get_sysconf_files();

    if files.is_empty() {
        return Ok(());
    }

    let mut watcher = Watcher::new(files, POLL_INTERVAL).await?;
    let detector = Detector::new(platform);

    let mut last_state = detector.check().await;

    loop {
        let mut changed = watcher.check().await.unwrap_or(false);
        let current_state = detector.check().await;

        if last_state == State::Draft && current_state == State::Applied {
            changed = true;
        }

        last_state = current_state;

        if changed {
            // TODO: Upload
        }

        watcher.tick().await;
    }
}
