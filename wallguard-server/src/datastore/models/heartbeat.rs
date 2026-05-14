use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HeartbeatModel {
    pub device_id: String,
}

impl HeartbeatModel {
    pub fn from_device_id(device_id: String) -> Self {
        Self { device_id }
    }
}
