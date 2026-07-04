//! Persists the arguments last used with `wallguard-cli start`, so that
//! after the first successful start, `control-channel-url`, `platform` and
//! `batch-size` no longer need to be repeated on the command line — any
//! flag omitted on a later `start` reuses its cached value, and any flag
//! passed overwrites the cache for next time.

use crate::arguments::Platform;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StartConfig {
    pub control_channel_url: Option<String>,
    pub platform: Option<Platform>,
    pub batch_size: Option<usize>,
}

fn config_path() -> PathBuf {
    wallguard_common::single_instance::state_dir().join("cli_start_config.json")
}

impl StartConfig {
    /// Reads the cached config, if any. Missing or unparsable files are
    /// treated as "nothing cached yet" rather than an error.
    pub fn load() -> Self {
        std::fs::read_to_string(config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}
