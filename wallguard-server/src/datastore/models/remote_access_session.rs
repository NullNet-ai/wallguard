use crate::{datastore::db_tables::DBTable, utilities::random::generate_random_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RemoteAccessType {
    Ssh,
    Tty,
    Ui,
    RemoteDesktop,
}

impl TryFrom<&str> for RemoteAccessType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lc_value = value.to_lowercase();
        match lc_value.as_str() {
            "ssh" => Ok(RemoteAccessType::Ssh),
            "tty" => Ok(RemoteAccessType::Tty),
            "ui" => Ok(RemoteAccessType::Ui),
            "remote_desktop" => Ok(RemoteAccessType::RemoteDesktop),
            _ => Err(format!("Remote access of type {lc_value} is not suppored")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAccessSession {
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
    pub fn new(
        device_id: impl Into<String>,
        instance_id: impl Into<String>,
        r#type: RemoteAccessType,
    ) -> Self {
        let token = generate_random_string(32).to_ascii_lowercase();

        Self {
            device_id: device_id.into(),
            instance_id: instance_id.into(),
            token,
            r#type,
            local_addr: None,
            local_port: None,
            protocol: None,
        }
    }

    pub fn set_ex_data(&mut self, addr: String, port: u32, protocol: String) {
        self.local_addr = Some(addr);
        self.local_port = Some(port);
        self.protocol = Some(protocol);
    }

    pub fn pluck() -> Vec<String> {
        vec![
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
