use crate::control_service::service::WallGuardService;
use crate::protocol::wallguard_commands::ControlChannelRequest;
use crate::protocol::wallguard_service::wall_guard_server::WallGuard;
use crate::token_provider::TokenProvider;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

impl WallGuardService {
    pub(crate) async fn control_channel_impl(
        &self,
        request: Request<ControlChannelRequest>,
    ) -> Result<Response<<WallGuardService as WallGuard>::ControlChannelStream>, Status> {
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

        let device = self
            .context
            .datastore
            .obtain_device_by_id(&token.jwt, &token.account.device.id)
            .await
            .map_err(|err| Status::internal(err.to_str()))?
            .ok_or(format!(
                "Unexpected error: no device found by id {}",
                &token.account.device.id
            ))
            .map_err(|err| Status::internal(err))?;

        if !device.authorized {
            let status = Status::invalid_argument("Device is not authorized");
            return Err(status);
        }

        if self
            .context
            .orchestractor
            .is_client_connected(&device.uuid)
            .await
        {
            let message = format!("Client for device '{}' is already connected", &device.uuid);
            log::error!("{message}");
            return Err(Status::internal(message));
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(32);

        self.context
            .orchestractor
            .on_client_connected(&device.uuid, token_provider, sender)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        Ok(Response::new(ReceiverStream::new(receiver)))
    }
}
