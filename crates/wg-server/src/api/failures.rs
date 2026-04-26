use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_shared::types::*;

use crate::{error::AppError, middleware::auth::RequestContext, AppState};

// ---------------------------------------------------------------------------
// Query params / response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct FailuresQuery {
    pub limit:    Option<i64>,
    pub offset:   Option<i64>,
    pub severity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FailuresResponse {
    pub items: Vec<AgentFailure>,
    pub total: i64,
}

// ---------------------------------------------------------------------------
// GET /api/v1/devices/{device_id}/failures
// ---------------------------------------------------------------------------

pub async fn list_failures(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(device_id): Path<Uuid>,
    Query(q): Query<FailuresQuery>,
) -> Result<Json<FailuresResponse>, AppError> {
    let limit  = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);

    // Validate severity filter if provided.
    if let Some(ref sev) = q.severity {
        match sev.as_str() {
            "warning" | "error" | "fatal" => {}
            _ => return Err(AppError::BadRequest(format!("invalid severity: {sev}"))),
        }
    }

    // The JOIN on d.org_id enforces org scoping without a separate pre-check.
    type Row = (
        Uuid,                        // failure_id
        Uuid,                        // device_id
        String,                      // severity
        String,                      // category
        String,                      // message
        Option<serde_json::Value>,   // context
        time::OffsetDateTime,        // occurred_at
        Option<time::OffsetDateTime>,// received_at
        bool,                        // is_replay
    );

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT df.failure_id, df.device_id, df.severity, df.category,
               df.message, df.context, df.occurred_at, df.received_at, df.is_replay
        FROM   device_failures df
        JOIN   devices d ON d.id = df.device_id
        WHERE  df.device_id = $1 AND d.org_id = $2
          AND  ($3::text IS NULL OR df.severity = $3)
        ORDER  BY df.occurred_at DESC
        LIMIT  $4 OFFSET $5
        "#,
    )
    .bind(device_id)
    .bind(ctx.org_id)
    .bind(&q.severity)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM   device_failures df
        JOIN   devices d ON d.id = df.device_id
        WHERE  df.device_id = $1 AND d.org_id = $2
          AND  ($3::text IS NULL OR df.severity = $3)
        "#,
    )
    .bind(device_id)
    .bind(ctx.org_id)
    .bind(&q.severity)
    .fetch_one(&state.pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|(failure_id, dev_id, sev, cat, msg, ctx_json, occurred_at, received_at, is_replay)| {
            AgentFailure {
                failure_id,
                device_id:   dev_id,
                severity:    parse_severity(&sev),
                category:    parse_category(&cat),
                message:     msg,
                context:     ctx_json,
                occurred_at: occurred_at.unix_timestamp() * 1000,
                received_at: received_at.map(|t| t.unix_timestamp() * 1000),
                is_replay,
            }
        })
        .collect();

    Ok(Json(FailuresResponse { items, total }))
}

// ---------------------------------------------------------------------------
// Parse helpers
// ---------------------------------------------------------------------------

fn parse_severity(s: &str) -> FailureSeverity {
    match s {
        "error" => FailureSeverity::Error,
        "fatal" => FailureSeverity::Fatal,
        _       => FailureSeverity::Warning,
    }
}

fn parse_category(s: &str) -> FailureCategory {
    match s {
        "tunnel"       => FailureCategory::Tunnel,
        "disk_buffer"  => FailureCategory::DiskBuffer,
        "fireparse"    => FailureCategory::Fireparse,
        "agent_crash"  => FailureCategory::AgentCrash,
        "connectivity" => FailureCategory::Connectivity,
        "system"       => FailureCategory::System,
        _              => FailureCategory::Monitoring,
    }
}
