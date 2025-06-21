use crate::{datastore::db_tables::DBTable, utilities};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceCredentials {
    pub id: String,
    pub account_id: String,
    pub account_secret: String,
    pub device_uuid: String,
}

impl DeviceCredentials {
    pub fn generate(device_uuid: String) -> Self {
        let account_id = utilities::random::generate_random_string(12);
        let account_secret = utilities::random::generate_random_string(32);
        Self {
            id: Default::default(),
            account_id,
            account_secret,
            device_uuid,
        }
    }

    pub fn pluck() -> Vec<String> {
        vec![
            "id".into(),
            "account_id".into(),
            "account_secret".into(),
            "device_uuid".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::DeviceCredentials
    }
}
