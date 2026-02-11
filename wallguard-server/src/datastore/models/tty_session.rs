use std::fmt;

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};

use crate::datastore::db_tables::DBTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TtySessionStatus {
    #[default]
    Active,
    Expired,
    Terminated,
}

impl TryFrom<&str> for TtySessionStatus {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "active" => Ok(TtySessionStatus::Active),
            "expired" => Ok(TtySessionStatus::Expired),
            "terminated" => Ok(TtySessionStatus::Terminated),
            other => Err(format!("Unexpected status {other}")).handle_err(location!()),
        }
    }
}

impl fmt::Display for TtySessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TtySessionStatus::Active => "active",
            TtySessionStatus::Expired => "expired",
            TtySessionStatus::Terminated => "terminated",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TtySessionModel {
    pub id: String,
    #[serde(rename = "device_tunnel_id")]
    pub tunnel_id: String,
    pub device_id: String,
    pub instance_id: String,
    pub session_status: TtySessionStatus,
    // @TODO
    // pub username: String,
}

impl TtySessionModel {
    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "device_tunnel_id".into(),
            "device_id".into(),
            "instance_id".into(),
            "session_status".into(),
            // "username".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::TtySessions
    }
}
