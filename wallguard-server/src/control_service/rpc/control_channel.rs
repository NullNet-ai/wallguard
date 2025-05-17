use nullnet_libtoken::Token;
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

        let jwt = self
            .context
            .datastore
            .login(&request.app_id, &request.app_secret)
            .await
            .map_err(|err| {
                let message = format!("Datastore request faield: {}", err.to_str());
                log::error!("{}", message);
                Status::internal(message)
            })?;

        let token = Token::from_jwt(&jwt).map_err(|_| {
            let message = "Invalid JWT: malformed token or wrong credentials";
            log::error!("{}", message);
            Status::internal(message)
        })?;

        let device_id = &token.account.device.id;

        if self
            .context
            .orchestractor
            .is_client_connected(device_id)
            .await
        {
            let message = format!("Client for device '{}' is already connected", device_id);
            log::error!("{message}");
            return Err(Status::internal(message));
        }

        let (_sender, receiver) = tokio::sync::mpsc::channel(6);

        Ok(Response::new(ReceiverStream::new(receiver)))
    }
}
