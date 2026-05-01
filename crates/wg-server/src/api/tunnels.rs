use axum::{
    extract::{Extension, Path, State},
    Json,
};
use opentelemetry::trace::TraceContextExt;
use serde::{Deserialize, Serialize};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;
use wg_shared::types::Role;

use crate::{error::AppError, middleware::auth::RequestContext, AppState};

// ---------------------------------------------------------------------------
// Response shape
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TunnelCreatedResponse {
    pub session_id: Uuid,
    pub ws_url:     String,
}

// ---------------------------------------------------------------------------
// Request bodies
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OpenSshRequest {
    pub username:   Option<String>,
    pub public_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenHttpRequest {
    pub target_host: String,
    pub target_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct OpenRdpRequest {
    pub width:       u32,
    pub height:      u32,
    pub target_fps:  u32,
    pub target_kbps: u32,
}

// ---------------------------------------------------------------------------
// RDP pending session params (stored after POST, consumed by WS handler)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RdpSessionParams {
    pub width:       u32,
    pub height:      u32,
    pub target_fps:  u32,
    pub target_kbps: u32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(crate) fn new_command_id() -> String {
    let trace_id = tracing::Span::current()
        .context()
        .span()
        .span_context()
        .trace_id()
        .to_string();
    format!("{trace_id}:{}", uuid::Uuid::new_v4())
}

async fn check_device_and_connected(
    state:     &AppState,
    device_id: Uuid,
    org_id:    Uuid,
) -> Result<(), AppError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT org_id FROM devices WHERE id = $1",
    )
    .bind(device_id)
    .fetch_optional(&state.pool)
    .await?;

    let (device_org_id,) = row.ok_or_else(|| AppError::NotFound(format!("device {device_id} not found")))?;

    if device_org_id != org_id {
        return Err(AppError::Forbidden);
    }

    let connected_ids = state.registry.connected_device_ids().await;
    if !connected_ids.contains(&device_id) {
        return Err(AppError::BadRequest(format!("device {device_id} is not connected")));
    }

    Ok(())
}

async fn insert_tunnel_session(
    state:       &AppState,
    session_id:  Uuid,
    device_id:   Uuid,
    tunnel_type: &str,
    initiated_by: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO tunnel_sessions (id, device_id, tunnel_type, initiated_by) VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(device_id)
    .bind(tunnel_type)
    .bind(initiated_by)
    .execute(&state.pool)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// POST /api/v1/devices/:id/tunnels/ssh
// ---------------------------------------------------------------------------

pub async fn open_ssh(
    State(state):  State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(device_id): Path<Uuid>,
    Json(_body):   Json<OpenSshRequest>,
) -> Result<Json<TunnelCreatedResponse>, AppError> {
    if !ctx.role.satisfies(Role::Operator) {
        return Err(AppError::Forbidden);
    }

    check_device_and_connected(&state, device_id, ctx.org_id).await?;

    let session_id = Uuid::new_v4();
    insert_tunnel_session(&state, session_id, device_id, "ssh", ctx.user_id).await?;

    Ok(Json(TunnelCreatedResponse {
        session_id,
        ws_url: format!("/api/v1/devices/{device_id}/tunnels/ssh/{session_id}"),
    }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/devices/:id/tunnels/tty
// ---------------------------------------------------------------------------

pub async fn open_tty(
    State(state):   State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<TunnelCreatedResponse>, AppError> {
    if !ctx.role.satisfies(Role::Operator) {
        return Err(AppError::Forbidden);
    }

    check_device_and_connected(&state, device_id, ctx.org_id).await?;

    let session_id = Uuid::new_v4();
    insert_tunnel_session(&state, session_id, device_id, "tty", ctx.user_id).await?;

    Ok(Json(TunnelCreatedResponse {
        session_id,
        ws_url: format!("/api/v1/devices/{device_id}/tunnels/tty/{session_id}"),
    }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/devices/:id/tunnels/http
// ---------------------------------------------------------------------------

pub async fn open_http(
    State(state):   State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(device_id): Path<Uuid>,
    Json(_body):    Json<OpenHttpRequest>,
) -> Result<Json<TunnelCreatedResponse>, AppError> {
    if !ctx.role.satisfies(Role::Operator) {
        return Err(AppError::Forbidden);
    }

    check_device_and_connected(&state, device_id, ctx.org_id).await?;

    let session_id = Uuid::new_v4();
    insert_tunnel_session(&state, session_id, device_id, "http", ctx.user_id).await?;

    Ok(Json(TunnelCreatedResponse {
        session_id,
        ws_url: format!("/api/v1/devices/{device_id}/tunnels/http/{session_id}"),
    }))
}

// ---------------------------------------------------------------------------
// POST /api/v1/devices/:id/tunnels/rdp
// ---------------------------------------------------------------------------

pub async fn open_rdp(
    State(state):    State<AppState>,
    Extension(ctx):  Extension<RequestContext>,
    Path(device_id): Path<Uuid>,
    Json(body):      Json<OpenRdpRequest>,
) -> Result<Json<TunnelCreatedResponse>, AppError> {
    if !ctx.role.satisfies(Role::Operator) {
        return Err(AppError::Forbidden);
    }

    check_device_and_connected(&state, device_id, ctx.org_id).await?;

    let session_id = Uuid::new_v4();
    insert_tunnel_session(&state, session_id, device_id, "rdp", ctx.user_id).await?;

    state.pending_rdp.lock().await.insert(session_id, RdpSessionParams {
        width:       body.width,
        height:      body.height,
        target_fps:  body.target_fps,
        target_kbps: body.target_kbps,
    });

    Ok(Json(TunnelCreatedResponse {
        session_id,
        ws_url: format!("/api/v1/devices/{device_id}/tunnels/rdp/{session_id}"),
    }))
}
