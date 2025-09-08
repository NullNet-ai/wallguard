use super::types::FileData;
use nullnet_liberror::Error;

mod config_xml;
mod nftables_ruleset;

pub use config_xml::*;
pub use nftables_ruleset::*;

pub trait FileToMonitor {
    fn take_snapshot(&self) -> FileData;
    async fn update(&mut self) -> Result<bool, Error>;
}

pub enum SystemConfigurationFile {
    ConfigXml(ConfigXml),
    NftablesRuleset(NftablesRuleset),
}

impl FileToMonitor for SystemConfigurationFile {
    fn take_snapshot(&self) -> FileData {
        match self {
            SystemConfigurationFile::ConfigXml(inner) => inner.take_snapshot(),
            SystemConfigurationFile::NftablesRuleset(inner) => inner.take_snapshot(),
        }
    }

    async fn update(&mut self) -> Result<bool, Error> {
        match self {
            SystemConfigurationFile::ConfigXml(inner) => inner.update().await,
            SystemConfigurationFile::NftablesRuleset(inner) => inner.update().await,
        }
    }
}
