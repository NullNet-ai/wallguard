use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::control_service::service::WallGuardService;
use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::protocol::wallguard_service::ControlChannelRequest;
use crate::protocol::wallguard_service::wall_guard_server::WallGuard;

#[tonic::async_trait]
impl WallGuard for WallGuardService {
    type ControlChannelStream = ReceiverStream<Result<WallGuardCommand, Status>>;

    async fn control_channel(
        &self,
        _request: Request<ControlChannelRequest>,
    ) -> Result<Response<Self::ControlChannelStream>, Status> {
        todo!()
    }
}
