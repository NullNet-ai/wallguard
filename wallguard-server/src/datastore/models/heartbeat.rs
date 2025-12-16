use crate::datastore::db_tables::DBTable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HeartbeatModel {
    pub device_id: String,
}

impl HeartbeatModel {
    pub fn pluck() -> Vec<String> {
        vec!["id".into(), "device_id".into()]
    }

    pub fn table() -> DBTable {
        DBTable::Heartbeats
    }

    pub fn from_device_id(device_id: String) -> Self {
        Self { device_id }
    }
}
