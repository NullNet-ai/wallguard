use serde::{Deserialize, Serialize};

use crate::datastore::db_tables::DBTable;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceConfiguration {
    pub id: String,
    pub digest: String,
    pub hostname: String,
    pub device_id: String,
    #[serde(rename = "config_version")]
    pub version: i32,
}

impl DeviceConfiguration {
    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "digest".into(),
            "hostname".into(),
            "device_id".into(),
            "config_version".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::DeviceConfigurations
    }
}
