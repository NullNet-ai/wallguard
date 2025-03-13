use nullnet_liberror::Error;
use tonic::{Request, Response};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{StatusRequest, StatusResponse},
};

impl WallGuardImpl {
    pub(crate) async fn device_status_impl(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Error> {
        let status_request = request.into_inner();
        let (jwt_token, token_info) = Self::authenticate(status_request.auth)?;

        let status = self
            .context
            .datastore
            .device_status(token_info.account.device.id, &jwt_token)
            .await?;

        Ok(Response::new(StatusResponse {
            status: status.into(),
        }))
    }
}
