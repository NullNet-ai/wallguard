use crate::protocol::wallguard_commands::{ClientMessage, ServerMessage};
use crate::protocol::wallguard_service::wall_guard_server::WallGuardServer;
use crate::{app_context::AppContext, protocol::wallguard_service::wall_guard_server::WallGuard};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::SocketAddr;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};

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
}
