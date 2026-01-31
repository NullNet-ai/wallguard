use crate::datastore::db_tables::DBTable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum RemoteAccessType {
    #[default]
    Ssh,
    Tty,
    Ui,
    RemoteDesktop,
    Mcp,
}

impl TryFrom<&str> for RemoteAccessType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lc_value = value.to_lowercase();
        match lc_value.as_str() {
            "ui" => Ok(RemoteAccessType::Ui),
            "ssh" => Ok(RemoteAccessType::Ssh),
            "tty" => Ok(RemoteAccessType::Tty),
            "mcp" => Ok(RemoteAccessType::Mcp),
            "remote_desktop" => Ok(RemoteAccessType::RemoteDesktop),
            _ => Err(format!("Remote access of type {lc_value} is not suppored")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteAccessSession {
    pub id: String,
    pub device_id: String,
    pub instance_id: String,
    #[serde(rename = "remote_access_session")]
    pub token: String,
    #[serde(rename = "remote_access_type")]
    pub r#type: RemoteAccessType,
    #[serde(rename = "remote_access_local_addr")]
    pub local_addr: Option<String>,
    #[serde(rename = "remote_access_local_port")]
    pub local_port: Option<u32>,
    #[serde(rename = "remote_access_local_protocol")]
    pub protocol: Option<String>,
}

impl RemoteAccessSession {
    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "device_id".into(),
            "remote_access_session".into(),
            "remote_access_type".into(),
            "instance_id".into(),
            "remote_access_local_addr".into(),
            "remote_access_local_port".into(),
            "remote_access_local_protocol".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::RemoteAccessSessions
    }
}
