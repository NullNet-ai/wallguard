use crate::control_service::service::WallGuardService;
use crate::token::Token;
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

        let device = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let resources_count = data.resources.len();
        log::info!("Received {} system resources", resources_count);

        if !data.resources.is_empty() {
            let start = std::time::Instant::now();
            self.context
                .datastore
                .create_system_resources(&token.jwt, data.resources, device.id)
                .await
                .map_err(|e| Status::internal(format!("Datastore operation failed: {e}")))?;
            log::info!(
                "create_system_resources: inserted {} records in {}ms",
                resources_count,
                start.elapsed().as_millis()
            );
        }

        Ok(Response::new(()))
    }
}
