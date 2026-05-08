use std::collections::HashSet;

use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use wg_shared::types::*;

use crate::{error::AppError, middleware::auth::RequestContext, AppState};

// ---------------------------------------------------------------------------
// Response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub items: Vec<Device>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct DeviceStatusResponse {
    pub device_id:    Uuid,
    pub connected:    bool,
    pub last_seen_at: Option<i64>,
}

// ---------------------------------------------------------------------------
// Parse helpers
// ---------------------------------------------------------------------------

fn parse_firewall_kind(s: &str) -> FirewallKind {
    match s {
        "pfsense"  => FirewallKind::PfSense,
        "opnsense" => FirewallKind::OPNSense,
        "nftables" => FirewallKind::NFTables,
        _          => FirewallKind::None,
    }
}

fn parse_feature(s: &str) -> Option<Feature> {
    match s {
        "network_monitoring"   => Some(Feature::NetworkMonitoring),
        "telemetry_monitoring" => Some(Feature::TelemetryMonitoring),
        "config_monitoring"    => Some(Feature::ConfigMonitoring),
        "ssh_tunnel"           => Some(Feature::SshTunnel),
        "tty_tunnel"           => Some(Feature::TtyTunnel),
        "http_tunnel"          => Some(Feature::HttpTunnel),
        "named_commands"       => Some(Feature::NamedCommands),
        "remote_desktop"       => Some(Feature::RemoteDesktop),
        _                      => None,
    }
}

// ---------------------------------------------------------------------------
// Row type for device queries
// ---------------------------------------------------------------------------

type DeviceRow = (
    Uuid,                         // id
    Uuid,                         // org_id
    String,                       // display_name
    String,                       // firewall_kind
    Option<String>,               // agent_version
    time::OffsetDateTime,         // enrolled_at
    Option<time::OffsetDateTime>, // last_seen_at
    Option<String>,               // config_digest
    Option<String>,               // notes
    Vec<String>,                  // features
    Option<serde_json::Value>,    // system_info
);

fn row_to_device(row: DeviceRow) -> Device {
    let (id, org_id, display_name, firewall_kind_str, agent_version,
         enrolled_at, last_seen_at, config_digest, notes, feature_strs, system_info_json) = row;

    let system_info = system_info_json
        .and_then(|v| serde_json::from_value(v).ok());

    Device {
        id,
        org_id,
        display_name,
        firewall_kind:  parse_firewall_kind(&firewall_kind_str),
        agent_version,
        features:       feature_strs.iter().filter_map(|s| parse_feature(s)).collect(),
        enrolled_at:    enrolled_at.unix_timestamp() * 1000,
        last_seen_at:   last_seen_at.map(|t| t.unix_timestamp() * 1000),
        config_digest,
        notes,
        system_info,
    }
}

// ---------------------------------------------------------------------------
// GET /api/v1/devices
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Json<DeviceListResponse>, AppError> {
    let rows = sqlx::query_as::<_, DeviceRow>(
        r#"
        SELECT id, org_id, display_name, firewall_kind, agent_version,
               enrolled_at, last_seen_at, config_digest, notes, features, system_info
        FROM   devices
        WHERE  org_id = $1
        ORDER  BY display_name
        "#,
    )
    .bind(ctx.org_id)
    .fetch_all(&state.pool)
    .await?;

    let total = rows.len() as i64;

    let _connected: HashSet<Uuid> = state
        .registry
        .connected_device_ids()
        .await
        .into_iter()
        .collect();

    let items = rows.into_iter().map(row_to_device).collect();

    Ok(Json(DeviceListResponse { items, total }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/devices/:id
// ---------------------------------------------------------------------------

pub async fn get_one(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<Device>, AppError> {
    let row = sqlx::query_as::<_, DeviceRow>(
        r#"
        SELECT id, org_id, display_name, firewall_kind, agent_version,
               enrolled_at, last_seen_at, config_digest, notes, features, system_info
        FROM   devices
        WHERE  id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound(format!("device {id} not found")))?;

    if row.1 != ctx.org_id {
        return Err(AppError::Forbidden);
    }

    Ok(Json(row_to_device(row)))
}

// ---------------------------------------------------------------------------
// GET /api/v1/devices/:id/status
// ---------------------------------------------------------------------------

pub async fn status(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeviceStatusResponse>, AppError> {
    // Verify device belongs to the org.
    let row: Option<(Uuid, Option<time::OffsetDateTime>)> = sqlx::query_as(
        "SELECT org_id, last_seen_at FROM devices WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let (org_id, last_seen_at) = row.ok_or_else(|| AppError::NotFound(format!("device {id} not found")))?;

    if org_id != ctx.org_id {
        return Err(AppError::Forbidden);
    }

    let connected_ids: HashSet<Uuid> = state
        .registry
        .connected_device_ids()
        .await
        .into_iter()
        .collect();

    let connected = connected_ids.contains(&id);

    Ok(Json(DeviceStatusResponse {
        device_id:    id,
        connected,
        last_seen_at: last_seen_at.map(|t| t.unix_timestamp() * 1000),
    }))
}
