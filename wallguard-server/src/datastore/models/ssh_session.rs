use std::fmt;

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};

use crate::datastore::db_tables::DBTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SshSessionStatus {
    #[default]
    Active,
    Expired,
    Terminated,
}

impl TryFrom<&str> for SshSessionStatus {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "active" => Ok(SshSessionStatus::Active),
            "expired" => Ok(SshSessionStatus::Expired),
            "terminated" => Ok(SshSessionStatus::Terminated),
            other => Err(format!("Unexpected status {other}")).handle_err(location!()),
        }
    }
}

impl fmt::Display for SshSessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SshSessionStatus::Active => "active",
            SshSessionStatus::Expired => "expired",
            SshSessionStatus::Terminated => "terminated",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SshSessionModel {
    pub id: String,
    #[serde(rename = "device_tunnel_id")]
    pub tunnel_id: String,
    pub device_id: String,
    pub instance_id: String,
    pub local_addr: String,
    pub local_port: u16,
    pub session_status: SshSessionStatus,
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
    pub username: String,
}

impl SshSessionModel {
    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "device_tunnel_id".into(),
            "device_id".into(),
            "instance_id".into(),
            "local_addr".into(),
            "local_port".into(),
            "session_status".into(),
            "public_key".into(),
            "private_key".into(),
            "passphrase".into(),
            "username".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::SshSessions
    }
}
