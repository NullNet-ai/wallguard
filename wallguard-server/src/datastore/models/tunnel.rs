use std::fmt::Display;

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};

use crate::datastore::db_tables::DBTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    #[default]
    Tty,
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
            "tty" => Ok(TunnelType::Tty),
            other => {
                Err(format!("Tunnel of type {other} is not supported")).handle_err(location!())
            }
        }
    }
}

impl Display for TunnelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TunnelType::Tty => "tty",
            TunnelType::Ssh => "ssh",
            TunnelType::Http => "http",
            TunnelType::Https => "https",
        };

        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    #[default]
    Active,
    Terminated,
}

impl TryFrom<&str> for TunnelStatus {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "active" => Ok(TunnelStatus::Active),
            "terminated" => Ok(TunnelStatus::Terminated),
            other => Err(format!("Unexpected tunnel status {other}")).handle_err(location!()),
        }
    }
}

impl Display for TunnelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TunnelStatus::Active => "active",
            TunnelStatus::Terminated => "terminated",
        };

        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TunnelModel {
    pub id: String,
    pub device_id: String,
    pub tunnel_type: TunnelType,
    pub service_id: String,
    pub tunnel_status: TunnelStatus,
    pub last_accessed: u64,
}

impl TunnelModel {
    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "device_id".into(),
            "tunnel_type".into(),
            "service_id".into(),
            "tunnel_status".into(),
            "last_accessed".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::DeviceTunnels
    }
}
