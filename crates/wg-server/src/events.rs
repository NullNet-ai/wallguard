use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SseEvent {
    pub org_id: Uuid,
    pub kind:   SseEventKind,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEventKind {
    DeviceConnected    { device_id: Uuid },
    DeviceDisconnected { device_id: Uuid },
    NewFailure         { device_id: Uuid, failure_id: Uuid, severity: String },
}
