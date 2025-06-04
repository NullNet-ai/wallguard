use crate::control_service::WallGuardService;
use crate::datastore::Device;
use crate::protocol::wallguard_authorization::authorization_status::State;
use crate::protocol::wallguard_authorization::{
    AuthorizationApproved, AuthorizationRequest, AuthorizationStatus,
};
use crate::protocol::wallguard_service::wall_guard_server::WallGuard;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

impl WallGuardService {
    pub(crate) async fn device_authorization_impl(
        &self,
        request: Request<AuthorizationRequest>,
    ) -> Result<Response<<WallGuardService as WallGuard>::DeviceAuthorizationStream>, Status> {
        let (sender, receiver) = tokio::sync::mpsc::channel(32);

        let request = request.into_inner();

        let org_id = request.organization_id;
        let device_uuid = request.device_uuid;

        let is_connected = self
            .context
            .orchestractor
            .is_client_connected(&device_uuid)
            .await;

        let is_authorizing = self
            .context
            .orchestractor
            .is_auth_pending(&device_uuid)
            .await;

        if is_connected || is_authorizing {
            let status = AuthorizationStatus {
                state: Some(State::Rejected(())),
            };

            log::warn!(
                "Rejecting auth: device {} is already connected or authorizing",
                device_uuid
            );

            sender
                .send(Ok(status))
                .await
                .map_err(|err| Status::internal(err.to_string()))?;

            return Ok(Response::new(ReceiverStream::new(receiver)));
        }

        let systoken = self
            .context
            .token_provider
            .get()
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let device = self
            .context
            .datastore
            .obtain_device_by_uuid(&systoken.jwt, &device_uuid)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        if device.is_none() {
            let mut device = Device::default();

            device.authorized = false;
            device.uuid = device_uuid;

            self.context
                .datastore
                .create_device(&systoken.jwt, &device, Some(org_id))
                .await
                .map_err(|err| Status::internal(err.to_str()))?;

            self.context
                .orchestractor
                .on_client_requested_authorization(&device.uuid, sender)
                .await
                .map_err(|err| Status::internal(err.to_str()))?;

            Ok(Response::new(ReceiverStream::new(receiver)))
        } else {
            let device = device.unwrap();

            if !device.authorized {
                self.context
                    .orchestractor
                    .on_client_requested_authorization(&device_uuid, sender)
                    .await
                    .map_err(|err| Status::internal(err.to_str()))?;

                Ok(Response::new(ReceiverStream::new(receiver)))
            } else {
                let status = AuthorizationStatus {
                    state: Some(State::Approved(AuthorizationApproved {
                        app_id: None,
                        app_secret: None,
                    })),
                };

                sender
                    .send(Ok(status))
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;

                Ok(Response::new(ReceiverStream::new(receiver)))
            }
        }
    }
}
