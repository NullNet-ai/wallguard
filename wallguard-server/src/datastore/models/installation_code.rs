use crate::datastore::db_tables::DBTable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationCode {
    pub id: String,
    pub device_id: String,
    pub device_code: String,
    pub redeemed: bool,
    pub organization_id: String,
}

impl InstallationCode {
    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "redeemed".into(),
            "device_id".into(),
            "device_code".into(),
            "organization_id".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::InstallationCodes
    }
}
