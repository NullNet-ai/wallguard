use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{CommonResponse, SetupRequest},
};

impl WallGuardImpl {
    pub(crate) async fn setup_impl(
        &self,
        request: Request<SetupRequest>,
    ) -> Result<Response<CommonResponse>, Status> {
        let datastore = self
            .datastore
            .as_ref()
            .ok_or_else(|| Status::internal("Datastore is unavailable"))?;

        let remote_address = request
            .remote_addr()
            .map_or_else(|| "Unknown".to_string(), |addr| addr.ip().to_string());

        let setup_request = request.into_inner();

        let jwt_token = setup_request
            .auth
            .ok_or_else(|| Status::internal("Unauthorized request"))?
            .token;

        let token_info =
            Token::from_jwt(&jwt_token).map_err(|e| Status::internal(e.to_string()))?;

        let response = datastore
            .device_setup(
                jwt_token,
                token_info.account.device.id,
                setup_request.device_version,
                setup_request.device_uuid,
                setup_request.hostname,
                remote_address,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CommonResponse {
            success: response.success,
            message: response.message,
        }))
    }
}
