use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_shared::types::{InstallationCode, Role};

use crate::{
    error::AppError,
    middleware::auth::RequestContext,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateInstallationCodeRequest {
    /// TTL in hours (default 24, max 720 = 30 days).
    pub ttl_hours: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub offset: Option<i64>,
    pub limit:  Option<i64>,
    /// If true, include already-used codes.
    pub include_used: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ListInstallationCodesResponse {
    pub items: Vec<InstallationCodeRow>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct InstallationCodeRow {
    pub code:       String,
    pub org_id:     Uuid,
    pub created_by: Uuid,
    pub used:       bool,
    pub expires_at: i64,   // Unix ms
    pub created_at: i64,   // Unix ms
}

/// `POST /api/v1/installation-codes`  (Admin+)
pub async fn create_installation_code(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(body): Json<CreateInstallationCodeRequest>,
) -> Result<(StatusCode, Json<InstallationCodeRow>), AppError> {
    ctx.require_role(Role::Admin).map_err(|_| AppError::Forbidden)?;

    let ttl_hours = body.ttl_hours.unwrap_or(24).clamp(1, 720);
    let expires_at = time::OffsetDateTime::now_utc()
        + time::Duration::hours(ttl_hours);

    // Generate a short, URL-safe random code.
    let code = generate_code();

    sqlx::query(
        r#"
        INSERT INTO installation_codes
            (code, org_id, created_by, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&code)
    .bind(ctx.org_id)
    .bind(ctx.user_id)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    let row = InstallationCodeRow {
        code,
        org_id:     ctx.org_id,
        created_by: ctx.user_id,
        used:       false,
        expires_at: (expires_at.unix_timestamp()) * 1000,
        created_at: time::OffsetDateTime::now_utc().unix_timestamp() * 1000,
    };

    Ok((StatusCode::CREATED, Json(row)))
}

/// `GET /api/v1/installation-codes`  (Admin+)
pub async fn list_installation_codes(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListInstallationCodesResponse>, AppError> {
    ctx.require_role(Role::Admin).map_err(|_| AppError::Forbidden)?;

    let limit        = q.limit.unwrap_or(50).clamp(1, 200);
    let offset       = q.offset.unwrap_or(0).max(0);
    let include_used = q.include_used.unwrap_or(false);

    let rows = sqlx::query_as::<_, (String, Uuid, Uuid, Option<time::OffsetDateTime>, time::OffsetDateTime, time::OffsetDateTime)>(
        r#"
        SELECT code, org_id, created_by, used_at, expires_at, created_at
        FROM   installation_codes
        WHERE  org_id = $1
          AND  ($2 OR used_at IS NULL)
          AND  expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT  $3 OFFSET $4
        "#,
    )
    .bind(ctx.org_id)
    .bind(include_used)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM installation_codes
        WHERE  org_id = $1
          AND  ($2 OR used_at IS NULL)
          AND  expires_at > NOW()
        "#,
    )
    .bind(ctx.org_id)
    .bind(include_used)
    .fetch_one(&state.pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|(code, org_id, created_by, used_at, expires_at, created_at)| {
            InstallationCodeRow {
                code,
                org_id,
                created_by,
                used:       used_at.is_some(),
                expires_at: expires_at.unix_timestamp() * 1000,
                created_at: created_at.unix_timestamp() * 1000,
            }
        })
        .collect();

    Ok(Json(ListInstallationCodesResponse { items, total }))
}

fn generate_code() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
