use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};
use tonic::Streaming;
use wallguard_common::protobuf::wallguard_commands::{ClientMessage, ServerMessage};
use wallguard_common::protobuf::wallguard_service::{
    ConfigSnapshot, ConnectionsData, DeviceSettingsRequest, DeviceSettingsResponse,
    SystemResourcesData,
};
use wallguard_common::wallguard_interface::WallGuardGrpcInterface;

#[derive(Debug, Clone)]
pub struct WGServer {
    interface: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    addr: SocketAddr,
}

impl WGServer {
    pub fn new(addr: SocketAddr) -> Self {
        let interface = Default::default();
        Self { interface, addr }
    }

    pub async fn is_connected(&self) -> bool {
        self.interface.lock().await.is_some()
    }

    pub async fn connect(&self) -> Result<(), Error> {
        let mut lock = self.interface.lock().await;

        if lock.is_none() {
            let interface = WallGuardGrpcInterface::from_sockaddr(self.addr).await?;
            *lock = Some(interface);
        }

        Ok(())
    }

    pub async fn reset(&self) {
        let mut lock = self.interface.lock().await;
        *lock = None;
    }

    pub async fn get_interface(&self) -> Result<WallGuardGrpcInterface, Error> {
        const MAX_RETRIES: usize = 3;
        const RETRY_DELAY: Duration = Duration::from_secs(5);
        let mut attempt: usize = 0;

        loop {
            if !self.is_connected().await
                && let Err(e) = self.connect().await
            {
                if attempt >= MAX_RETRIES {
                    return Err(e);
                }

                attempt += 1;
                log::warn!("Failed to connect to the server (attempt {attempt}/{MAX_RETRIES}), retrying in {}s", RETRY_DELAY.as_secs());
                tokio::time::sleep(RETRY_DELAY).await;
                continue;
            }

            let lock = self.interface.lock().await;
            if let Some(interface) = &*lock {
                return Ok(interface.clone());
            }

            if attempt >= MAX_RETRIES {
                return Err("Failed to connect to the server").handle_err(location!());
            }

            attempt += 1;
            tokio::time::sleep(RETRY_DELAY).await;
        }
    }

    pub async fn request_control_channel(
        &self,
        receiver: mpsc::Receiver<ClientMessage>,
    ) -> Result<Streaming<ServerMessage>, Error> {
        self.get_interface()
            .await?
            .request_control_channel(receiver)
            .await
    }

    pub async fn handle_connections_data(&self, data: ConnectionsData) -> Result<(), Error> {
        self.get_interface()
            .await?
            .handle_connections_data(data)
            .await
    }

    pub async fn handle_system_resources_data(
        &self,
        data: SystemResourcesData,
    ) -> Result<(), Error> {
        self.get_interface()
            .await?
            .handle_system_resources_data(data)
            .await
    }

    pub async fn get_device_settings(
        &self,
        request: DeviceSettingsRequest,
    ) -> Result<DeviceSettingsResponse, Error> {
        self.get_interface()
            .await?
            .get_device_settings(request)
            .await
    }

    pub async fn handle_config_data(&self, data: ConfigSnapshot) -> Result<(), Error> {
        self.get_interface()
            .await?
            .handle_config_data(data)
            .await
    }
}
