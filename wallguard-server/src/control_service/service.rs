use crate::app_context::AppContext;
use crate::traffic_handler::ip_info::ip_info_handler;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};
use wallguard_common::protobuf::wallguard_commands::{ClientMessage, ServerMessage};
use wallguard_common::protobuf::wallguard_service::wall_guard_server::{
    WallGuard, WallGuardServer,
};
use wallguard_common::protobuf::wallguard_service::{
    ConfigSnapshot, DeviceSettingsRequest, DeviceSettingsResponse, PacketsData, SystemResourcesData,
};

// @TODO: Configure through ENV
const IP_INFO_CACHE_SIZE: usize = 10_000;

#[derive(Debug)]
pub struct WallGuardService {
    pub(crate) context: AppContext,
    pub(crate) ip_info_tx: mpsc::Sender<Option<IpAddr>>,
}

impl WallGuardService {
    pub fn new(context: AppContext) -> Self {
        let (ip_info_tx, ip_info_rx) = mpsc::channel();

        let handle = tokio::runtime::Handle::current();
        let ctx = context.clone();
        std::thread::spawn(move || {
            ip_info_handler(&ip_info_rx, IP_INFO_CACHE_SIZE, &handle, ctx);
        });

        Self {
            context,
            ip_info_tx,
        }
    }

    pub async fn serve(self, addr: SocketAddr) -> Result<(), Error> {
        Server::builder()
            .add_service(WallGuardServer::new(self))
            .serve(addr)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}

#[tonic::async_trait]
impl WallGuard for WallGuardService {
    type ControlChannelStream = ReceiverStream<Result<ServerMessage, Status>>;

    async fn control_channel(
        &self,
        request: Request<Streaming<ClientMessage>>,
    ) -> Result<Response<Self::ControlChannelStream>, Status> {
        log::debug!(
            "WallGuardService::control_channel requested from addr {}",
            request
                .remote_addr()
                .map(|addr| addr.to_string())
                .unwrap_or("unknown".into())
        );

        self.control_channel_impl(request).await
    }

    async fn handle_packets_data(
        &self,
        request: Request<PacketsData>,
    ) -> Result<Response<()>, Status> {
        self.handle_packets_data_impl(request).await
    }

    async fn handle_system_resources_data(
        &self,
        request: Request<SystemResourcesData>,
    ) -> Result<Response<()>, Status> {
        self.handle_system_resources_data_impl(request).await
    }

    async fn get_device_settings(
        &self,
        request: Request<DeviceSettingsRequest>,
    ) -> Result<Response<DeviceSettingsResponse>, Status> {
        self.get_device_settings_impl(request).await
    }

    async fn handle_config_data(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<()>, Status> {
        self.handle_config_data_impl(request).await
    }
}
