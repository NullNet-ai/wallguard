use super::super::types::FileData;
use crate::{data_transmission::sysconfig::data::FileToMonitor, utilities};
use nullnet_liberror::{location, Error, ErrorHandler, Location};

const FILE_PATH: &str = "/conf/config.xml";

#[derive(Debug, Default, Clone)]
pub struct ConfigXml {
    content: String,
}

impl FileToMonitor for ConfigXml {
    fn take_snapshot(&self) -> FileData {
        FileData {
            filename: "config.xml".into(),
            content: self.content.as_bytes().into(),
        }
    }

    async fn update(&mut self) -> Result<bool, Error> {
        let prev = utilities::hash::sha256_digest_bytes(&self.content);
        let content = tokio::fs::read(FILE_PATH).await.handle_err(location!())?;

        self.content = String::from_utf8_lossy(content.as_slice()).into();
        let curr = utilities::hash::sha256_digest_bytes(&self.content);

        Ok(prev != curr)
    }
}
