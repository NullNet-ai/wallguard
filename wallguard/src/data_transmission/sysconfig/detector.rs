use crate::client_data::Platform;

use std::ffi::OsStr;
use std::path::Path;
use tokio::fs;
use tokio::fs::ReadDir;

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
            Platform::OpnSense => todo!("Not implemented"),
            Platform::Generic => unreachable!(),
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
            {
                if ext == "dirty" {
                    return State::Draft;
                }
            }
        }

        State::Applied
    }
}
