use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::ApiResult;

#[derive(Debug, Clone, Deserialize)]
pub struct TunnelResponse {
    pub session_id: Uuid,
    pub ws_url:     String,
}

#[derive(Serialize)]
struct EmptyBody {}

#[derive(Serialize)]
struct HttpTunnelRequest<'a> {
    target_host: &'a str,
    target_port: u16,
}

#[derive(Serialize)]
struct RdpRequest {
    width:       u32,
    height:      u32,
    target_fps:  u32,
    target_kbps: u32,
}

/// POST /api/v1/devices/{device_id}/tunnels/ssh
pub async fn open_ssh(device_id: Uuid) -> ApiResult<TunnelResponse> {
    super::post(
        &format!("/api/v1/devices/{device_id}/tunnels/ssh"),
        &EmptyBody {},
    )
    .await
}

/// POST /api/v1/devices/{device_id}/tunnels/tty
pub async fn open_tty(device_id: Uuid) -> ApiResult<TunnelResponse> {
    super::post(
        &format!("/api/v1/devices/{device_id}/tunnels/tty"),
        &EmptyBody {},
    )
    .await
}

/// POST /api/v1/devices/{device_id}/tunnels/http
pub async fn open_http(
    device_id: Uuid,
    target_host: &str,
    target_port: u16,
) -> ApiResult<TunnelResponse> {
    super::post(
        &format!("/api/v1/devices/{device_id}/tunnels/http"),
        &HttpTunnelRequest { target_host, target_port },
    )
    .await
}

/// POST /api/v1/devices/{device_id}/tunnels/rdp
pub async fn open_rdp(
    device_id:   Uuid,
    width:       u32,
    height:      u32,
    target_fps:  u32,
    target_kbps: u32,
) -> ApiResult<TunnelResponse> {
    super::post(
        &format!("/api/v1/devices/{device_id}/tunnels/rdp"),
        &RdpRequest { width, height, target_fps, target_kbps },
    )
    .await
}
