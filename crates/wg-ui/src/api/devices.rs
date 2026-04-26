use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_shared::types::Device;
use super::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceListResponse {
    pub items: Vec<Device>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatusResponse {
    pub device_id:    Uuid,
    pub connected:    bool,
    pub last_seen_at: Option<i64>,
}

/// GET /api/v1/devices
pub async fn list() -> ApiResult<DeviceListResponse> {
    super::get("/api/v1/devices").await
}

/// GET /api/v1/devices/{id}
pub async fn get(id: Uuid) -> ApiResult<Device> {
    super::get(&format!("/api/v1/devices/{id}")).await
}

/// GET /api/v1/devices/{id}/status
pub async fn status(id: Uuid) -> ApiResult<DeviceStatusResponse> {
    super::get(&format!("/api/v1/devices/{id}/status")).await
}
