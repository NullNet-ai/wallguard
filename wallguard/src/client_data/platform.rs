use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Generic,
    PfSense,
    OpnSense,
    NfTables,
}

impl TryFrom<&str> for Platform {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "generic" => Ok(Platform::Generic),
            "pfsense" => Ok(Platform::PfSense),
            "opnsense" => Ok(Platform::OpnSense),
            "nftables" => Ok(Platform::NfTables),
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
        };

        write!(f, "{value}")
    }
}

impl Platform {
    pub fn can_monitor_config(&self) -> bool {
        !matches!(self, Platform::Generic | Platform::NfTables)
    }

    pub fn can_monitor_telemetry(&self) -> bool {
        true
    }

    pub fn can_monitor_traffic(&self) -> bool {
        true
    }

    pub fn get_sysconf_files(&self) -> Vec<PathBuf> {
        match self {
            Platform::PfSense | Platform::OpnSense => vec![PathBuf::from("/conf/config.xml")],
            Platform::Generic | Platform::NfTables => vec![],
        }
    }
}
