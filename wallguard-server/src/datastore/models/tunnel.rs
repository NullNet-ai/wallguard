use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};

use crate::datastore::db_tables::DBTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    #[default]
    Ssh,
    Http,
    Https,
}

impl TryFrom<&str> for TunnelType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "ssh" => Ok(TunnelType::Ssh),
            "http" => Ok(TunnelType::Http),
            "https" => Ok(TunnelType::Https),
            other => {
                Err(format!("Tunnel of type {other} is not supported")).handle_err(location!())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TunnelModel {
    pub id: String,
    pub device_id: String,
    pub tunnel_type: TunnelType,
    pub service_id: String,
}

impl TunnelModel {
    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "device_id".into(),
            "tunnel_type".into(),
            "service_id".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::DeviceTunnels
    }
}
