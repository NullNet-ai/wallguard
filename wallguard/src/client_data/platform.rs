use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::fmt;

use crate::data_transmission::sysconfig::data::{
    ConfigXml, NftablesRuleset, SystemConfigurationFile,
};

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Generic,
    PfSense,
    OpnSense,
    NfTables,
    Desktop,
}

impl TryFrom<&str> for Platform {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "generic" => Ok(Platform::Generic),
            "pfsense" => Ok(Platform::PfSense),
            "opnsense" => Ok(Platform::OpnSense),
            "nftables" => Ok(Platform::NfTables),
            "desktop" => Ok(Platform::Desktop),
            _ => {
                let errmsg = format!("Unsupported platform {value}");
                Err(errmsg).handle_err(location!())
            }
        }
    }
}

impl TryFrom<String> for Platform {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Platform::PfSense => "pfsense",
            Platform::OpnSense => "opnsense",
            Platform::Generic => "generic",
            Platform::NfTables => "nftables",
            Platform::Desktop => "desktop",
        };

        write!(f, "{value}")
    }
}

impl Platform {
    pub fn can_monitor_config(&self) -> bool {
        !matches!(self, Platform::Generic | Platform::Desktop)
    }

    pub fn can_monitor_telemetry(&self) -> bool {
        true
    }

    pub fn can_monitor_traffic(&self) -> bool {
        true
    }

    #[cfg(not(target_os = "freebsd"))]
    pub fn can_open_remote_desktop_session(&self) -> bool {
        matches!(self, Platform::Desktop)
    }

    pub fn get_sysconf_files(&self) -> Vec<SystemConfigurationFile> {
        match self {
            Platform::PfSense | Platform::OpnSense => {
                let file = ConfigXml::default();
                vec![SystemConfigurationFile::ConfigXml(file)]
            }
            Platform::NfTables => {
                let file = NftablesRuleset::default();
                vec![SystemConfigurationFile::NftablesRuleset(file)]
            }
            Platform::Generic | Platform::Desktop => vec![],
        }
    }
}
