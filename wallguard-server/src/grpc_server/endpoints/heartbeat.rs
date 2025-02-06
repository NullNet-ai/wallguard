use nullnet_libtoken::Token;
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

        let jwt_token = heartbeat_request
            .auth
            .ok_or_else(|| Status::internal("Unauthorized request"))?
            .token;

        let token_info =
            Token::from_jwt(&jwt_token).map_err(|e| Status::internal(e.to_string()))?;

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
