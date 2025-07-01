use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{
    ClientMessage, ConfigSnapshot, DeviceSettingsRequest, DeviceSettingsResponse, PacketsData,
    ServerMessage, SystemResourcesData, WallGuardGrpcInterface,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tonic::Streaming;

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

    async fn get_interface(&self) -> Result<WallGuardGrpcInterface, Error> {
        let max_retries = 3;
        let retry_delay = Duration::from_secs(5);

        for attempt in 0..=max_retries {
            if !self.is_connected().await {
                if let Err(e) = self.connect().await {
                    if attempt == max_retries {
                        return Err(e);
                    }

                    tokio::time::sleep(retry_delay).await;
                    continue;
                }
            }

            let lock = self.interface.lock().await;
            if let Some(interface) = &*lock {
                return Ok(interface.clone());
            }

            if attempt == max_retries {
                return Err("Failed to connect to the server").handle_err(location!());
            }

            tokio::time::sleep(retry_delay).await;
        }

        Err("Failed to connect to the server").handle_err(location!())
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

    pub async fn handle_packets_data(&self, data: PacketsData) -> Result<(), Error> {
        self.get_interface().await?.handle_packets_data(data).await
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
        self.get_interface().await?.handle_config_data(data).await
    }
}
