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
pub struct MetricsQuery {
    #[serde(default = "default_window")]
    pub window: String,
    #[serde(default = "default_bucket")]
    pub bucket: String,
}

fn default_window() -> String { "1h".into() }
fn default_bucket() -> String { "1m".into() }

#[derive(Debug, Serialize)]
pub struct MetricsPoint {
    pub t:               i64,
    pub cpu_percent:     f32,
    pub mem_used_bytes:  i64,
    pub mem_total_bytes: i64,
    pub disk_used_bytes: i64,
    pub disk_total_bytes: i64,
    pub load_1m:         f32,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub points: Vec<MetricsPoint>,
    pub window: String,
    pub bucket: String,
}

fn validate_interval(s: &str) -> bool {
    matches!(
        s,
        "10s" | "30s" | "1m" | "2m" | "5m" | "10m" | "15m" | "30m"
            | "1h" | "2h" | "6h" | "12h" | "24h" | "7d"
    )
}

// ---------------------------------------------------------------------------
// GET /api/v1/devices/:id/metrics
// ---------------------------------------------------------------------------

pub async fn get_metrics(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Query(q): Query<MetricsQuery>,
) -> Result<Json<MetricsResponse>, AppError> {
    if !validate_interval(&q.window) {
        return Err(AppError::BadRequest(format!("invalid window: {}", q.window)));
    }
    if !validate_interval(&q.bucket) {
        return Err(AppError::BadRequest(format!("invalid bucket: {}", q.bucket)));
    }

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

    let sql = format!(
        r#"
        SELECT
            time_bucket('{bucket}'::interval, time)    AS t,
            AVG(cpu_percent)::REAL                     AS cpu_percent,
            AVG(mem_used_bytes)::BIGINT                AS mem_used_bytes,
            MAX(mem_total_bytes)::BIGINT               AS mem_total_bytes,
            AVG(disk_used_bytes)::BIGINT               AS disk_used_bytes,
            MAX(disk_total_bytes)::BIGINT              AS disk_total_bytes,
            AVG(load_1m)::REAL                         AS load_1m
        FROM   resource_metrics
        WHERE  device_id = $1
          AND  time >= NOW() - '{window}'::interval
        GROUP  BY t
        ORDER  BY t
        "#,
        bucket = q.bucket,
        window = q.window,
    );

    let rows: Vec<(time::OffsetDateTime, f32, i64, i64, i64, i64, f32)> =
        sqlx::query_as(&sql)
            .bind(id)
            .fetch_all(&state.pool)
            .await?;

    let points = rows
        .into_iter()
        .map(|(t, cpu, mem_used, mem_total, disk_used, disk_total, load_1m)| MetricsPoint {
            t:                t.unix_timestamp() * 1000,
            cpu_percent:      cpu,
            mem_used_bytes:   mem_used,
            mem_total_bytes:  mem_total,
            disk_used_bytes:  disk_used,
            disk_total_bytes: disk_total,
            load_1m,
        })
        .collect();

    Ok(Json(MetricsResponse {
        points,
        window: q.window,
        bucket: q.bucket,
    }))
}
