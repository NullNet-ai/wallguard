use serde::{Deserialize, Serialize};

use crate::datastore::db_tables::DBTable;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInstance {
    pub id: String,
    pub device_id: String,
}

impl DeviceInstance {
    pub fn pluck() -> Vec<String> {
        vec!["id".into(), "device_id".into()]
    }

    pub fn table() -> DBTable {
        DBTable::DeviceInstances
    }
}
