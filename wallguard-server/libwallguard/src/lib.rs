use nullnet_liberror::{Error, ErrorHandler, Location, location};

pub use proto::wallguard_commands::*;
pub use proto::wallguard_service::*;

use proto::wallguard_service::wall_guard_client::WallGuardClient;

use std::time::Duration;
use tokio::sync::mpsc;
use tonic::Request;
pub use tonic::Streaming;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;

mod proto;

#[derive(Clone, Debug)]
pub struct WallGuardGrpcInterface {
    client: WallGuardClient<Channel>,
}

impl WallGuardGrpcInterface {
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(addr: &str, port: u16) -> Result<Self, Error> {
        let addr = format!("http://{addr}:{port}");

        let channel = Channel::from_shared(addr)
            .expect("Failed to parse address")
            .timeout(Duration::from_secs(10))
            .keep_alive_timeout(Duration::from_secs(10))
            .connect()
            .await
            .handle_err(location!())?;

        let client = WallGuardClient::new(channel).max_decoding_message_size(50 * 1024 * 1024);

        Ok(Self { client })
    }

    pub async fn request_control_channel(
        &self,
        receiver: mpsc::Receiver<ClientMessage>,
    ) -> Result<Streaming<ServerMessage>, Error> {
        let receiver = ReceiverStream::new(receiver);

        let response = self
            .client
            .clone()
            .control_channel(Request::new(receiver))
            .await
            .handle_err(location!())?;

        Ok(response.into_inner())
    }

    pub async fn handle_packets_data(&self, data: PacketsData) -> Result<(), Error> {
        self.client
            .clone()
            .handle_packets_data(Request::new(data))
            .await
            .handle_err(location!())
            .map(|response| response.into_inner())
    }

    pub async fn handle_system_resources_data(
        &self,
        data: SystemResourcesData,
    ) -> Result<(), Error> {
        self.client
            .clone()
            .handle_system_resources_data(Request::new(data))
            .await
            .handle_err(location!())
            .map(|response| response.into_inner())
    }

    pub async fn get_device_settings(
        &self,
        request: DeviceSettingsRequest,
    ) -> Result<DeviceSettingsResponse, Error> {
        self.client
            .clone()
            .get_device_settings(request)
            .await
            .handle_err(location!())
            .map(|response| response.into_inner())
    }

    pub async fn handle_config_data(&self, request: ConfigSnapshot) -> Result<(), Error> {
        self.client
            .clone()
            .handle_config_data(request)
            .await
            .handle_err(location!())
            .map(|response| response.into_inner())
    }
}
