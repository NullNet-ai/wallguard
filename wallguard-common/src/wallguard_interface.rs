use nullnet_liberror::{Error, ErrorHandler, Location, location};

use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::Streaming;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Channel, Endpoint};

use crate::protobuf::wallguard_commands::{ClientMessage, ServerMessage};
use crate::protobuf::wallguard_service::ServicesMessage;
use crate::protobuf::wallguard_service::wall_guard_client::WallGuardClient;
use crate::protobuf::wallguard_service::{
    ConfigSnapshot, ConnectionsData, DeviceSettingsRequest, DeviceSettingsResponse,
    SystemResourcesData,
};

/// `Endpoint::timeout` only bounds individual requests made over an
/// already-established channel; it does not bound the initial TCP/TLS
/// handshake done by `connect()`. Without this, a connection attempt that
/// gets silently dropped (rather than actively refused) can hang forever.
async fn connect_with_timeout(ep: Endpoint) -> Result<Channel, Error> {
    let ep = configure_endpoint(ep);
    match tokio::time::timeout(Duration::from_secs(10), ep.connect()).await {
        Ok(result) => result.handle_err(location!()),
        Err(_) => Err("Timed out connecting to the server").handle_err(location!()),
    }
}

/// `keep_alive_timeout` on its own is inert: it is the deadline for a ping
/// ack, and pings are only sent once `http2_keep_alive_interval` is set.
/// Without pings (or TCP keepalive) a connection that dies without a FIN/RST
/// goes unnoticed until the kernel's retransmission timeout, if it is
/// writing at all, and forever if it is idle.
fn configure_endpoint(ep: Endpoint) -> Endpoint {
    ep.connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .http2_keep_alive_interval(Duration::from_secs(30))
        .keep_alive_timeout(Duration::from_secs(10))
        .keep_alive_while_idle(true)
        .timeout(Duration::from_secs(10))
}

#[derive(Clone, Debug)]
pub struct WallGuardGrpcInterface {
    client: WallGuardClient<Channel>,
}

impl WallGuardGrpcInterface {
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(addr: &str, port: u16) -> Result<Self, Error> {
        let addr = format!("http://{addr}:{port}");

        let channel = {
            let ep = Channel::from_shared(addr).expect("Failed to parse address");
            connect_with_timeout(ep).await?
        };

        let client = WallGuardClient::new(channel).max_decoding_message_size(50 * 1024 * 1024);

        Ok(Self { client })
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn from_sockaddr(addr: SocketAddr) -> Result<Self, Error> {
        let addr = format!("http://{addr}");

        let channel = {
            let ep = Channel::from_shared(addr).expect("Failed to parse address");
            connect_with_timeout(ep).await?
        };

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

    pub async fn handle_connections_data(&self, data: ConnectionsData) -> Result<(), Error> {
        self.client
            .clone()
            .handle_connections_data(Request::new(data))
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

    pub async fn report_services(&self, request: ServicesMessage) -> Result<(), Error> {
        self.client
            .clone()
            .report_services(request)
            .await
            .handle_err(location!())
            .map(|response| response.into_inner())
    }
}
