use axum::{
    extract::{Extension, State},
    http::{header, HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_shared::types::Role;

use crate::{
    auth::{password, refresh},
    error::AppError,
    middleware::auth::RequestContext,
    AppState,
};

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token:  String,
    pub token_type:    &'static str,
    pub expires_in:    u64,
    pub refresh_token: String,
}

// ---------------------------------------------------------------------------
// POST /api/v1/auth/login
// ---------------------------------------------------------------------------

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
        "SELECT id, org_id, password_hash, role FROM users WHERE email = $1",
    )
    .bind(&body.email)
    .fetch_optional(&state.pool)
    .await?;

    let (user_id, org_id, password_hash, role_str) =
        row.ok_or(AppError::Unauthorized)?;

    let ok = password::verify_password(&body.password, &password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !ok {
        return Err(AppError::Unauthorized);
    }

    let role = parse_role(&role_str);

    let access_token = state.jwt
        .issue(user_id, org_id, role)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let refresh_token = refresh::issue_refresh_token(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: 3600,
        refresh_token,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/auth/logout
// ---------------------------------------------------------------------------

pub async fn logout(
    State(state): State<AppState>,
    Extension(_ctx): Extension<RequestContext>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = state.jwt
        .validate(token)
        .await
        .map_err(|_| AppError::Unauthorized)?;

    state.jwt
        .revoke(&claims)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/auth/refresh
// ---------------------------------------------------------------------------

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let (new_refresh, user_id, org_id, role) =
        refresh::rotate_refresh_token(&state.pool, &body.refresh_token)
            .await
            .map_err(|_| AppError::Unauthorized)?;

    let access_token = state.jwt
        .issue(user_id, org_id, role)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: 3600,
        refresh_token: new_refresh,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_role(s: &str) -> Role {
    match s {
        "owner"    => Role::Owner,
        "admin"    => Role::Admin,
        "operator" => Role::Operator,
        _          => Role::Viewer,
    }
}
