use crate::control_service::service::WallGuardService;
use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};
use wallguard_common::protobuf::wallguard_service::SystemResourcesData;

impl WallGuardService {
    pub(crate) async fn handle_system_resources_data_impl(
        &self,
        request: Request<SystemResourcesData>,
    ) -> Result<Response<()>, Status> {
        let data = request.into_inner();

        let token =
            Token::from_jwt(&data.token).map_err(|_| Status::internal("Malformed JWT token"))?;

        let _ = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        log::info!("Received {} system resources.", data.resources.len());

        if !data.resources.is_empty() {
            self.context
                .datastore
                .create_system_resources(&token.jwt, data.resources)
                .await
                .map_err(|_| Status::internal("Datastore operation failed"))?;
        }

        Ok(Response::new(()))
    }
}
