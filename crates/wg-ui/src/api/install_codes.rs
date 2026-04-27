use serde::{Deserialize, Serialize};

use super::{post, get, ApiResult};

#[derive(Debug, Clone, Deserialize)]
pub struct InstallationCodeRow {
    pub code:       String,
    pub used:       bool,
    pub expires_at: i64,   // Unix ms
    pub created_at: i64,   // Unix ms
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListInstallationCodesResponse {
    pub items: Vec<InstallationCodeRow>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateInstallationCodeRequest {
    pub ttl_hours: Option<i64>,
}

pub async fn list() -> ApiResult<ListInstallationCodesResponse> {
    get("/api/v1/installation-codes").await
}

pub async fn create(ttl_hours: Option<i64>) -> ApiResult<InstallationCodeRow> {
    post("/api/v1/installation-codes", &CreateInstallationCodeRequest { ttl_hours }).await
}
