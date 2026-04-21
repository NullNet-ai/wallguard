// HTTP API request/response types.
// Defined here so wg-server and wg-ui share identical serialization.
// All types compile to both native and wasm32.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{
    AgentFailure, ConfigDrift, Device, DeviceStatus, Feature, FirewallKind,
    InstallationCode, Role, TunnelType, User,
};

// ---------------------------------------------------------------------------
// Error envelope
// ---------------------------------------------------------------------------

/// Every non-2xx response body has this shape:
/// `{ "error": { "code": "...", "message": "...", "request_id": "..." } }`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    /// Machine-readable error code, e.g. `"device_not_found"`.
    pub code:       String,
    pub message:    String,
    pub request_id: Option<String>,
}

impl ApiErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { error: ApiErrorDetail { code: code.into(), message: message.into(), request_id: None } }
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.error.request_id = Some(id.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items:  Vec<T>,
    pub total:  u64,
    pub offset: u64,
    pub limit:  u64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, offset: u64, limit: u64) -> Self {
        Self { items, total, offset, limit }
    }

    pub fn single_page(items: Vec<T>) -> Self {
        let total = items.len() as u64;
        Self { items, total, offset: 0, limit: total }
    }
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token:  String,
    pub refresh_token: String,
    pub expires_in:    u64,  // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub type RefreshResponse = LoginResponse;

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email:        String,
    pub display_name: String,
    pub password:     String,
    pub role:         Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub role:         Option<Role>,
}

pub type UserResponse = User;

// ---------------------------------------------------------------------------
// Devices
// ---------------------------------------------------------------------------

pub type DeviceResponse = Device;

/// Lighter summary used in list views; omits large/optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSummary {
    pub id:            Uuid,
    pub display_name:  String,
    pub firewall_kind: FirewallKind,
    pub agent_version: Option<String>,
    pub connected:     bool,
    pub degraded:      bool,
    pub last_seen_at:  Option<i64>,
    pub features:      Vec<Feature>,
}

// ---------------------------------------------------------------------------
// Installation codes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstallationCodeRequest {
    /// Optional expiry in seconds from now. Defaults to 24 hours.
    pub expires_in_secs: Option<u64>,
}

pub type InstallationCodeResponse = InstallationCode;

// ---------------------------------------------------------------------------
// Tunnel open requests / response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSshTunnelRequest {
    pub username:   String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTtyTunnelRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenHttpTunnelRequest {
    pub target_host: String,
    pub target_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRemoteDesktopRequest {
    pub width:      u32,
    pub height:     u32,
    pub target_fps: u32,
    /// Target video bitrate in kbps.
    pub target_kbps: u32,
}

/// Returned by all tunnel-open endpoints on success.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTunnelResponse {
    pub session_id:    Uuid,
    pub tunnel_type:   TunnelType,
    /// Relative WebSocket path: `/api/v1/devices/{id}/{type}/{session_id}`
    pub websocket_path: String,
}

// ---------------------------------------------------------------------------
// Server-Sent Events
// ---------------------------------------------------------------------------

/// Discriminant field included in every SSE `data` payload so clients can
/// dispatch without relying solely on the SSE `event:` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SseEventKind {
    DeviceConnected,
    DeviceDisconnected,
    DeviceStatus,
    NewFailure,
    ConfigDrift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseDeviceConnected {
    pub kind:          SseEventKind,
    pub device_id:     Uuid,
    pub agent_version: Option<String>,
    pub occurred_at:   i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseDeviceDisconnected {
    pub kind:        SseEventKind,
    pub device_id:   Uuid,
    pub occurred_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseDeviceStatus {
    pub kind:        SseEventKind,
    pub status:      DeviceStatus,
    pub occurred_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseNewFailure {
    pub kind:        SseEventKind,
    pub failure:     AgentFailure,
    pub occurred_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfigDrift {
    pub kind:        SseEventKind,
    pub drift:       ConfigDrift,
    pub occurred_at: i64,
}
