use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{StatusRequest, StatusResponse},
    utils::map_status_value_to_enum,
};

impl WallGuardImpl {
    pub(crate) async fn device_status_impl(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let status_request = request.into_inner();
        let (jwt_token, token_info) = Self::authenticate(status_request.auth)?;

        let status = self
            .datastore
            .device_status(token_info.account.device.id, jwt_token)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StatusResponse {
            status: map_status_value_to_enum(&status).into(),
        }))
    }
}
