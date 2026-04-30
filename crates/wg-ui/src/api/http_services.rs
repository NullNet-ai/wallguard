use serde::Deserialize;
use uuid::Uuid;

use super::ApiResult;

#[derive(Debug, Clone, Deserialize)]
pub struct HttpServiceRow {
    pub port:   u32,
    pub scheme: String,
    pub title:  String,
}

pub async fn list(device_id: Uuid) -> ApiResult<Vec<HttpServiceRow>> {
    super::get(&format!("/api/v1/devices/{device_id}/http-services")).await
}
