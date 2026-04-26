use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_shared::types::*;

use crate::{
    auth::password,
    error::AppError,
    middleware::auth::RequestContext,
    AppState,
};

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit:  Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub items: Vec<User>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email:        String,
    pub display_name: String,
    pub password:     String,
    pub role:         Role,
}

// ---------------------------------------------------------------------------
// GET /api/v1/users  (Admin+)
// ---------------------------------------------------------------------------

pub async fn list_users(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<ListQuery>,
) -> Result<Json<UsersResponse>, AppError> {
    ctx.require_role(Role::Admin).map_err(|_| AppError::Forbidden)?;

    let limit  = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);

    type Row = (Uuid, Uuid, String, String, String, time::OffsetDateTime);

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT id, org_id, email, display_name, role, created_at
        FROM   users
        WHERE  org_id = $1
        ORDER  BY created_at
        LIMIT  $2 OFFSET $3
        "#,
    )
    .bind(ctx.org_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE org_id = $1",
    )
    .bind(ctx.org_id)
    .fetch_one(&state.pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|(id, org_id, email, display_name, role_str, created_at)| User {
            id,
            org_id,
            email,
            display_name,
            role:       parse_role(&role_str),
            created_at: created_at.unix_timestamp() * 1000,
        })
        .collect();

    Ok(Json(UsersResponse { items, total }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/users  (Admin+)
// ---------------------------------------------------------------------------

pub async fn create_user(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    ctx.require_role(Role::Admin).map_err(|_| AppError::Forbidden)?;

    // Callers may not create a role at or above their own level.
    if body.role >= ctx.role {
        return Err(AppError::Forbidden);
    }

    let password_hash = password::hash_password(&body.password)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let id = Uuid::new_v4();

    let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, time::OffsetDateTime)>(
        r#"
        INSERT INTO users (id, org_id, email, display_name, password_hash, role)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, org_id, email, display_name, role, created_at
        "#,
    )
    .bind(id)
    .bind(ctx.org_id)
    .bind(&body.email)
    .bind(&body.display_name)
    .bind(&password_hash)
    .bind(role_str(body.role))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint().is_some() {
                return AppError::Conflict("email already in use".into());
            }
        }
        AppError::from(e)
    })?;

    let (uid, org_id, email, display_name, role_s, created_at) = row;
    let user = User {
        id: uid,
        org_id,
        email,
        display_name,
        role:       parse_role(&role_s),
        created_at: created_at.unix_timestamp() * 1000,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/users/{id}  (Admin+)
// ---------------------------------------------------------------------------

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ctx.require_role(Role::Admin).map_err(|_| AppError::Forbidden)?;

    if id == ctx.user_id {
        return Err(AppError::BadRequest("cannot delete yourself".into()));
    }

    let result = sqlx::query(
        "DELETE FROM users WHERE id = $1 AND org_id = $2",
    )
    .bind(id)
    .bind(ctx.org_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("user {id} not found")));
    }

    Ok(StatusCode::NO_CONTENT)
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

fn role_str(r: Role) -> &'static str {
    match r {
        Role::Owner    => "owner",
        Role::Admin    => "admin",
        Role::Operator => "operator",
        Role::Viewer   => "viewer",
    }
}
