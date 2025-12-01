use crate::client_data::Platform;

use std::ffi::OsStr;
use std::path::Path;
use tokio::fs;
use tokio::fs::ReadDir;
use wallguard_common::protobuf::wallguard_service::ConfigStatus;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    Draft,
    Applied,
    Undefined,
}

pub struct Detector {
    platform: Platform,
}

impl Detector {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }

    pub async fn check(&self) -> State {
        match &self.platform {
            Platform::PfSense => Detector::check_pfsense().await,
            Platform::OpnSense => Detector::check_opnsense().await,
            Platform::NfTables => State::Applied,
            Platform::Generic | Platform::Desktop => unreachable!(),
        }
    }

    async fn check_pfsense() -> State {
        let mut entries: ReadDir = match fs::read_dir("/var/run/").await {
            Ok(entries) => entries,
            Err(_) => return State::Undefined,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(ext) = Path::new(&entry.file_name())
                .extension()
                .and_then(OsStr::to_str)
                && ext == "dirty"
            {
                return State::Draft;
            }
        }

        State::Applied
    }

    async fn check_opnsense() -> State {
        let mut entries: ReadDir = match fs::read_dir("/var/tmp/").await {
            Ok(entries) => entries,
            Err(_) => return State::Undefined,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(ext) = Path::new(&entry.file_name())
                .extension()
                .and_then(OsStr::to_str)
                && ext == "dirty"
            {
                return State::Draft;
            }
        }

        State::Applied
    }
}

#[allow(clippy::from_over_into)]
impl Into<i32> for State {
    fn into(self) -> i32 {
        match self {
            State::Draft => ConfigStatus::CsDraft.into(),
            State::Applied => ConfigStatus::CsApplied.into(),
            State::Undefined => ConfigStatus::CsUndefined.into(),
        }
    }
}
