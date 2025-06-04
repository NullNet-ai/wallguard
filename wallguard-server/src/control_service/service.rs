use crate::protocol::wallguard_authorization::{AuthorizationRequest, AuthorizationStatus};
use crate::protocol::wallguard_commands::{ControlChannelRequest, WallGuardCommand};
use crate::protocol::wallguard_service::wall_guard_server::WallGuardServer;
use crate::{app_context::AppContext, protocol::wallguard_service::wall_guard_server::WallGuard};

use std::net::SocketAddr;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use nullnet_liberror::{Error, ErrorHandler, Location, location};

#[derive(Debug)]
pub struct WallGuardService {
    pub(crate) context: AppContext,
}

impl WallGuardService {
    pub fn new(context: AppContext) -> Self {
        Self { context }
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
    type DeviceAuthorizationStream = ReceiverStream<Result<AuthorizationStatus, Status>>;

    async fn device_authorization(
        &self,
        request: Request<AuthorizationRequest>,
    ) -> Result<Response<<WallGuardService as WallGuard>::DeviceAuthorizationStream>, Status> {
        self.device_authorization_impl(request).await
    }

    type ControlChannelStream = ReceiverStream<Result<WallGuardCommand, Status>>;

    async fn control_channel(
        &self,
        request: Request<ControlChannelRequest>,
    ) -> Result<Response<<WallGuardService as WallGuard>::ControlChannelStream>, Status> {
        self.control_channel_impl(request).await
    }
}
