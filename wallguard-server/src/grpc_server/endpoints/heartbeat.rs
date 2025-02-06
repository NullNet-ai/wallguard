use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{CommonResponse, HeartbeatRequest},
};

impl WallGuardImpl {
    pub(crate) async fn heartbeat_impl(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<CommonResponse>, Status> {
        let heartbeat_request = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(heartbeat_request.auth)?;

        let response = self
            .datastore
            .as_ref()
            .ok_or_else(|| Status::internal("Datastore is unavailable"))?
            .heartbeat(&jwt_token, token_info.account.device.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CommonResponse {
            success: response.success,
            message: response.message,
        }))
    }
}
