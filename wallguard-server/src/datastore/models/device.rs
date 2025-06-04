use crate::datastore::db_tables::DBTable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Device {
    #[serde(rename = "device_uuid")]
    pub uuid: String,
    #[serde(rename = "is_traffic_monitoring_enabled")]
    pub traffic_monitoring: bool,
    #[serde(rename = "is_config_monitoring_enabled")]
    pub sysconf_monitoring: bool,
    #[serde(rename = "is_telemetry_monitoring_enabled")]
    pub telemetry_monitoring: bool,
    #[serde(rename = "is_device_authorized")]
    pub authorized: bool,
}

impl Device {
    pub fn pluck() -> Vec<String> {
        vec![
            "device_uuid".into(),
            "is_traffic_monitoring_enabled".into(),
            "is_config_monitoring_enabled".into(),
            "is_telemetry_monitoring_enabled".into(),
            "is_device_authorized".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::Devices
    }

    pub fn entity_prefix() -> String {
        "DV".into()
    }
}
