use crate::app_context::AppContext;
use crate::datastore::{ServiceInfo, TunnelModel, TunnelType};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TunnelCreateError {
    DeviceNotFound,
    DeviceNotAuthorized,
    ServiceNotFound,
    WrongDeviceForService,
    UnsupportedProtocol,
    CanEstablishATunnel,
    DatastoreError,
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
            Self::CanEstablishATunnel => "failed to establish a tunnel",
        };

        f.write_str(msg)
    }
}

impl std::error::Error for TunnelCreateError {}

#[derive(Debug, Clone)]
pub struct TunnelCommonData {
    pub(crate) service_data: ServiceInfo,
    pub(crate) tunnel_data: TunnelModel,
    context: AppContext,
}

impl TunnelCommonData {
    pub async fn create(
        context: AppContext,
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
            ..Default::default()
        };

        let id = context
            .datastore
            .create_tunnel(&jwt, &tunnel_data)
            .await
            .map_err(|_| TunnelCreateError::DatastoreError)?;

        tunnel_data.id = id;

        Ok(Self {
            service_data,
            tunnel_data,
            context,
        })
    }
}
