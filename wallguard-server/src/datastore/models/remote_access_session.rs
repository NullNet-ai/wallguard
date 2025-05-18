use crate::{datastore::db_tables::DBTable, utilities::random::generate_random_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RemoteAccessType {
    Ssh,
    Tty,
    Ui,
}

impl TryFrom<&str> for RemoteAccessType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lc_value = value.to_lowercase();
        match lc_value.as_str() {
            "ssh" => Ok(RemoteAccessType::Ssh),
            "tty" => Ok(RemoteAccessType::Tty),
            "ui" => Ok(RemoteAccessType::Ui),
            _ => Err(format!(
                "Remote access of type {} is not suppored",
                lc_value
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAccessSession {
    pub device_id: String,
    #[serde(rename = "remote_access_session")]
    pub token: String,
    #[serde(rename = "remote_access_type")]
    pub r#type: RemoteAccessType,
}

impl RemoteAccessSession {
    pub fn new(device_id: impl Into<String>, r#type: RemoteAccessType) -> Self {
        let token = generate_random_string(16);

        Self {
            device_id: device_id.into(),
            token,
            r#type,
        }
    }

    pub fn pluck() -> Vec<String> {
        vec![
            "device_id".into(),
            "remote_access_session".into(),
            "remote_access_type".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::RemoteAccessSessions
    }

    pub fn entity_prefix() -> String {
        "RA".into()
    }
}
