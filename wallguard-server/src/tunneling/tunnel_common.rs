use nullnet_liberror::Error;
use tokio::sync::Mutex;

use crate::app_context::AppContext;
use crate::datastore::{ServiceInfo, TunnelModel, TunnelStatus, TunnelType};
use crate::tunneling::http::HttpTunnel;
use crate::tunneling::ssh::SshTunnel;
use crate::tunneling::tty::TtyTunnel;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TunnelCreateError {
    DeviceNotFound,
    DeviceNotAuthorized,
    ServiceNotFound,
    WrongDeviceForService,
    UnsupportedProtocol,
    CantEstablishATunnel,
    DatastoreError,
    SshKeygenError,
    SshSessionFailed,
}

impl Display for TunnelCreateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::DeviceNotFound => "device not found",
            Self::DeviceNotAuthorized => "device not authorized",
            Self::ServiceNotFound => "service not found",
            Self::WrongDeviceForService => "service does not belong to device",
            Self::UnsupportedProtocol => "unsupported protocol",
            Self::DatastoreError => "datastore error",
            Self::CantEstablishATunnel => "failed to establish a tunnel",
            Self::SshKeygenError => "ssh key generation failed",
            Self::SshSessionFailed => "failed to create ssh session",
        };

        f.write_str(msg)
    }
}

impl TunnelCreateError {
    pub fn to_http_status(self) -> u16 {
        match self {
            Self::DeviceNotFound => 404,
            Self::DeviceNotAuthorized => 401,
            Self::ServiceNotFound => 404,
            Self::WrongDeviceForService => 400,
            Self::UnsupportedProtocol => 400,
            Self::CantEstablishATunnel => 503,
            Self::DatastoreError => 500,
            Self::SshKeygenError => 500,
            Self::SshSessionFailed => 500,
        }
    }
}

impl std::error::Error for TunnelCreateError {}

#[derive(Debug, Clone)]
pub struct TunnelCommonData {
    pub(crate) service_data: ServiceInfo,
    pub(crate) tunnel_data: TunnelModel,
}

impl TunnelCommonData {
    pub async fn create(
        context: Arc<AppContext>,
        jwt: &str,
        device_id: &str,
        service_id: &str,
    ) -> Result<Self, TunnelCreateError> {
        let _ = context
            .datastore
            .obtain_device_by_id(jwt, device_id, false)
            .await
            .map_err(|_| TunnelCreateError::DatastoreError)?
            .ok_or(TunnelCreateError::DeviceNotFound)
            .and_then(|device| {
                if device.authorized {
                    Ok(device)
                } else {
                    Err(TunnelCreateError::DeviceNotAuthorized)
                }
            })?;

        let service_data = context
            .datastore
            .obtain_service(jwt, service_id, false)
            .await
            .map_err(|_| TunnelCreateError::DatastoreError)?
            .ok_or(TunnelCreateError::ServiceNotFound)
            .and_then(|service| {
                if service.device_id == device_id {
                    Ok(service)
                } else {
                    Err(TunnelCreateError::WrongDeviceForService)
                }
            })?;

        let tunnel_type = TunnelType::try_from(service_data.protocol.as_str())
            .map_err(|_| TunnelCreateError::UnsupportedProtocol)?;

        let mut tunnel_data = TunnelModel {
            device_id: device_id.into(),
            service_id: service_id.into(),
            tunnel_type,
            tunnel_status: match tunnel_type {
                TunnelType::Tty | TunnelType::Ssh => TunnelStatus::Idle,
                _ => TunnelStatus::Active,
            },
            created_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ..Default::default()
        };

        let id = context
            .datastore
            .create_tunnel(jwt, &tunnel_data)
            .await
            .map_err(|_| TunnelCreateError::DatastoreError)?;

        tunnel_data.id = id;

        Ok(Self {
            service_data,
            tunnel_data,
        })
    }
}

#[derive(Debug, Clone)]
pub enum WallguardTunnel {
    Http(Arc<Mutex<HttpTunnel>>),
    Ssh(Arc<Mutex<SshTunnel>>),
    Tty(Arc<Mutex<TtyTunnel>>),
}

impl WallguardTunnel {
    pub async fn terminate(self) -> Result<(), Error> {
        match self {
            WallguardTunnel::Http(http_tunnel) => http_tunnel.lock().await.terminate().await,
            WallguardTunnel::Ssh(ssh_tunnel) => ssh_tunnel.lock().await.terminate().await,
            WallguardTunnel::Tty(tty_tunnel) => tty_tunnel.lock().await.terminate().await,
        }
    }

    pub async fn service_id(&self) -> String {
        match self {
            WallguardTunnel::Http(tun) => tun.lock().await.data.service_data.id.clone(),
            WallguardTunnel::Ssh(tun) => tun.lock().await.data.service_data.id.clone(),
            WallguardTunnel::Tty(tun) => tun.lock().await.data.service_data.id.clone(),
        }
    }

    pub async fn tunnel_id(&self) -> String {
        match self {
            WallguardTunnel::Http(tun) => tun.lock().await.data.tunnel_data.id.clone(),
            WallguardTunnel::Ssh(tun) => tun.lock().await.data.tunnel_data.id.clone(),
            WallguardTunnel::Tty(tun) => tun.lock().await.data.tunnel_data.id.clone(),
        }
    }
}
