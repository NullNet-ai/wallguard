use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInstance {
    pub id: String,
    pub device_id: String,
}
