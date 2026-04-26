use serde::{Deserialize, Serialize};
use super::ApiResult;

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token:  String,
    pub token_type:    String,
    pub expires_in:    u64,
    pub refresh_token: String,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    email:    &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    refresh_token: &'a str,
}

#[derive(Serialize)]
struct EmptyBody {}

/// POST /api/v1/auth/login
pub async fn login(email: &str, password: &str) -> ApiResult<TokenResponse> {
    super::post("/api/v1/auth/login", &LoginRequest { email, password }).await
}

/// POST /api/v1/auth/logout
pub async fn logout() -> ApiResult<()> {
    super::post("/api/v1/auth/logout", &EmptyBody {}).await
}

/// POST /api/v1/auth/refresh
pub async fn refresh(refresh_token: &str) -> ApiResult<TokenResponse> {
    super::post("/api/v1/auth/refresh", &RefreshRequest { refresh_token }).await
}
