use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::control_service::service::WallGuardService;
use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::protocol::wallguard_service::ControlChannelRequest;
use crate::protocol::wallguard_service::wall_guard_server::WallGuard;
use crate::token_provider::TokenProvider;

#[tonic::async_trait]
impl WallGuard for WallGuardService {
    type ControlChannelStream = ReceiverStream<Result<WallGuardCommand, Status>>;

    async fn control_channel(
        &self,
        request: Request<ControlChannelRequest>,
    ) -> Result<Response<Self::ControlChannelStream>, Status> {
        let request = request.into_inner();

        let token_provider = TokenProvider::new(
            request.app_id,
            request.app_secret,
            self.context.datastore.clone(),
        );

        let token = token_provider
            .get()
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

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

        let (sender, receiver) = tokio::sync::mpsc::channel(32);

        self.context
            .orchestractor
            .on_client_connected(device_id, token_provider, sender)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        Ok(Response::new(ReceiverStream::new(receiver)))
    }
}
