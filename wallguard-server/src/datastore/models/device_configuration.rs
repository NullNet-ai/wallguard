use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceConfiguration {
    pub id: String,
    pub digest: String,
    pub hostname: String,
    pub device_id: String,
    #[serde(rename = "config_version")]
    pub version: i32,
}
