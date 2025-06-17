use crate::token_provider::Token;
use tonic::{Request, Response, Status};

use crate::{
    control_service::service::WallGuardService, protocol::wallguard_service::SystemResourcesData,
};

impl WallGuardService {
    pub(crate) async fn handle_system_resources_data_impl(
        &self,
        request: Request<SystemResourcesData>,
    ) -> Result<Response<()>, Status> {
        let data = request.into_inner();

        let token =
            Token::from_jwt(&data.token).map_err(|_| Status::internal("Malformed JWT token"))?;

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
