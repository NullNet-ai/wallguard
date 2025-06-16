use std::path::PathBuf;

use nullnet_liberror::{location, Error, ErrorHandler, Location};
use target_os::TargetOs;

mod target_os;

#[derive(Debug, Clone, Copy, Default)]
pub enum Platform {
    #[default]
    Generic,
    PfSense,
    OpnSense,
}

impl TryFrom<&str> for Platform {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "generic" => Ok(Platform::Generic),
            "pfsense" => Ok(Platform::PfSense),
            "opnsense" => Ok(Platform::OpnSense),
            _ => {
                let errmsg = format!("Unsupported platform {}", value);
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

impl ToString for Platform {
    fn to_string(&self) -> String {
        let value = match self {
            Platform::PfSense => "pfsense",
            Platform::OpnSense => "opnsense",
            Platform::Generic => "generic",
        };

        value.to_string()
    }
}

impl Platform {
    pub fn get_target_os(&self) -> TargetOs {
        TargetOs::new()
    }

    pub fn can_monitor_config(&self) -> bool {
        match self {
            Platform::Generic => false,
            _ => true,
        }
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
            Platform::Generic => vec![],
        }
    }
}
