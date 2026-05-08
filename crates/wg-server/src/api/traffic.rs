use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::RequestContext, AppState};

// ---------------------------------------------------------------------------
// Query params + response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TrafficQuery {
    /// Time window, e.g. "1h", "6h", "24h"
    #[serde(default = "default_window")]
    pub window: String,
    /// Bucket size, e.g. "1m", "5m", "15m"
    #[serde(default = "default_bucket")]
    pub bucket: String,
}

fn default_window() -> String { "1h".into() }
fn default_bucket() -> String { "1m".into() }

#[derive(Debug, Serialize)]
pub struct TrafficPoint {
    /// Bucket start as Unix milliseconds
    pub t:         i64,
    pub out_bytes: i64,
    pub in_bytes:  i64,
}

#[derive(Debug, Serialize)]
pub struct TrafficResponse {
    pub points: Vec<TrafficPoint>,
    pub window: String,
    pub bucket: String,
}

// ---------------------------------------------------------------------------
// Validation helpers — both params are inlined into the SQL as intervals;
// restrict them to a safe whitelist to prevent injection.
// ---------------------------------------------------------------------------

fn validate_interval(s: &str) -> bool {
    matches!(
        s,
        "10s" | "30s" | "1m" | "2m" | "5m" | "10m" | "15m" | "30m"
            | "1h" | "2h" | "6h" | "12h" | "24h" | "7d"
    )
}

// ---------------------------------------------------------------------------
// GET /api/v1/devices/:id/traffic
// ---------------------------------------------------------------------------

pub async fn get_traffic(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Query(q): Query<TrafficQuery>,
) -> Result<Json<TrafficResponse>, AppError> {
    if !validate_interval(&q.window) {
        return Err(AppError::BadRequest(format!("invalid window: {}", q.window)));
    }
    if !validate_interval(&q.bucket) {
        return Err(AppError::BadRequest(format!("invalid bucket: {}", q.bucket)));
    }

    // Verify device belongs to caller's org.
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT org_id FROM devices WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let (org_id,) = row.ok_or_else(|| AppError::NotFound(format!("device {id} not found")))?;
    if org_id != ctx.org_id {
        return Err(AppError::Forbidden);
    }

    // The bucket and window strings are already validated against the whitelist
    // above, so it is safe to interpolate them directly into the query.
    let sql = format!(
        r#"
        SELECT
            time_bucket('{bucket}'::interval, time) AS t,
            COALESCE(SUM(bytes) FILTER (WHERE direction = 'out'), 0)::BIGINT AS out_bytes,
            COALESCE(SUM(bytes) FILTER (WHERE direction = 'in'),  0)::BIGINT AS in_bytes
        FROM   packets
        WHERE  device_id = $1
          AND  time >= NOW() - '{window}'::interval
        GROUP  BY t
        ORDER  BY t
        "#,
        bucket = q.bucket,
        window = q.window,
    );

    let rows: Vec<(time::OffsetDateTime, i64, i64)> = sqlx::query_as(&sql)
        .bind(id)
        .fetch_all(&state.pool)
        .await?;

    let points = rows
        .into_iter()
        .map(|(t, out_bytes, in_bytes)| TrafficPoint {
            t: t.unix_timestamp() * 1000,
            out_bytes,
            in_bytes,
        })
        .collect();

    Ok(Json(TrafficResponse {
        points,
        window: q.window,
        bucket: q.bucket,
    }))
}
