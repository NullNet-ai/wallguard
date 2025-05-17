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
        request: Request<ControlChannelRequest>,
    ) -> Result<Response<Self::ControlChannelStream>, Status> {
        let request = request.into_inner();

        let response = self
            .context
            .datastore
            .login(&request.app_id, &request.app_secret)
            .await
            .map_err(|err| {
                let message = format!("Datastore request faield: {}", err.to_str());
                Status::internal(message)
            })?;

        todo!()
    }
}