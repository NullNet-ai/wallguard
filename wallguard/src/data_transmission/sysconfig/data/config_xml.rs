use super::super::types::FileData;
use crate::{data_transmission::sysconfig::data::FileToMonitor, utilities};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

const FILE_PATH: &str = "/conf/config.xml";

#[derive(Debug, Default, Clone)]
pub struct ConfigXml {
    content: String,
    // Cached digest of `content`, so each poll only hashes the freshly-read
    // bytes once instead of re-hashing the (already-known) previous content
    // from scratch every 500ms tick just to compare it against itself.
    digest: [u8; 32],
}

impl FileToMonitor for ConfigXml {
    fn take_snapshot(&self) -> FileData {
        FileData {
            filename: "config.xml".into(),
            content: self.content.as_bytes().into(),
        }
    }

    async fn update(&mut self) -> Result<bool, Error> {
        let content = tokio::fs::read(FILE_PATH).await.handle_err(location!())?;
        self.content = String::from_utf8_lossy(content.as_slice()).into();

        let digest = utilities::hash::sha256_digest_bytes(&self.content);
        let changed = digest != self.digest;
        self.digest = digest;

        Ok(changed)
    }
}
