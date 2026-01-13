use crate::{control_service::service::WallGuardService, datastore::ServiceInfo};
use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};
use wallguard_common::protobuf::wallguard_service::ServicesMessage;

impl WallGuardService {
    pub(crate) async fn report_services_impl(
        &self,
        request: Request<ServicesMessage>,
    ) -> Result<Response<()>, Status> {
        let data = request.into_inner();

        let token =
            Token::from_jwt(&data.token).map_err(|_| Status::internal("Malformed JWT token"))?;

        let device = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let models: Vec<ServiceInfo> = data
            .services
            .into_iter()
            .map(|value| ServiceInfo::new(value, device.id.clone()))
            .collect();

        self.context
            .datastore
            .udpate_services(&token.jwt, &device.id, &models)
            .await
            .map_err(|_| Status::internal("Datastore operation failed"))?;

        Ok(Response::new(()))
    }
}
