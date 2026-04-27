use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_shared::types::Role;
use super::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id:    Uuid,
    pub email: String,
    pub name:  String,
    pub role:  Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    pub items: Vec<UserInfo>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub email:    String,
    pub name:     String,
    pub role:     Role,
    pub password: String,
}

/// GET /api/v1/users
pub async fn list() -> ApiResult<UsersResponse> {
    super::get("/api/v1/users").await
}

/// POST /api/v1/users
pub async fn create(req: CreateUserRequest) -> ApiResult<()> {
    super::post("/api/v1/users", &req).await
}

/// DELETE /api/v1/users/{id}
pub async fn delete(id: Uuid) -> ApiResult<()> {
    super::delete(&format!("/api/v1/users/{id}")).await
}
