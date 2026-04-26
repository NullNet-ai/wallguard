use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_shared::types::AgentFailure;
use super::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailuresResponse {
    pub items: Vec<AgentFailure>,
    pub total: i64,
}

/// GET /api/v1/devices/{device_id}/failures?offset={offset}&limit=20[&severity=...]
pub async fn list(
    device_id: Uuid,
    offset: u32,
    severity: Option<&str>,
) -> ApiResult<FailuresResponse> {
    let mut path = format!(
        "/api/v1/devices/{device_id}/failures?offset={offset}&limit=20"
    );
    if let Some(sev) = severity {
        path.push_str(&format!("&severity={sev}"));
    }
    super::get(&path).await
}
