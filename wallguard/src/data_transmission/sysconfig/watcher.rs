use super::types::FileInfo;
use super::utils::get_mtime;
use nullnet_liberror::Error;
use std::{path::PathBuf, time::Duration};

#[derive(Debug)]
pub struct Watcher {
    pub(super) files: Vec<FileInfo>,
    pub(super) poll_interval: Duration,
}

impl Watcher {
    pub async fn new(files: Vec<PathBuf>, poll_interval: Duration) -> Result<Self, Error> {
        let mut infos = Vec::new();

        for path in files {
            let mtime = get_mtime(&path).await?;
            infos.push(FileInfo { path, mtime });
        }

        Ok(Self {
            files: infos,
            poll_interval,
        })
    }

    pub async fn check(&mut self) -> Result<bool, Error> {
        let mut changed = false;

        for file in &mut self.files {
            let current = get_mtime(&file.path).await?;
            if current > file.mtime {
                file.mtime = current;
                changed = true;
            }
        }

        Ok(changed)
    }

    pub async fn tick(&self) {
        tokio::time::sleep(self.poll_interval).await
    }
}
