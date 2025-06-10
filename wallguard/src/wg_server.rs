use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{
    ClientMessage, DeviceSettingsRequest, DeviceSettingsResponse, PacketsData, ServerMessage,
    SystemResourcesData, WallGuardGrpcInterface,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tonic::Streaming;

#[derive(Debug, Clone, Default)]
pub struct WGServer {
    interface: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    addr: String,
    port: u16,
}

impl WGServer {
    pub fn new(addr: String, port: u16) -> Self {
        let interface = Default::default();
        Self {
            interface,
            addr,
            port,
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.interface.lock().await.is_some()
    }

    pub async fn connect(&self) -> Result<(), Error> {
        let mut lock = self.interface.lock().await;

        if lock.is_none() {
            let interface = WallGuardGrpcInterface::new(&self.addr, self.port).await?;
            *lock = Some(interface);
        }

        Ok(())
    }

    async fn get_interface(&self) -> Result<WallGuardGrpcInterface, Error> {
        if !self.is_connected().await {
            self.connect().await?;
        }

        let lock = self.interface.lock().await;
        match &*lock {
            Some(interface) => Ok(interface.clone()),
            None => Err("Interface unexpectedly None").handle_err(location!()),
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
}
