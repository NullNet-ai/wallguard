use nullnet_liberror::Error;
use tonic::{Request, Response};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{CommonResponse, SetupRequest},
};

impl WallGuardImpl {
    pub(crate) async fn setup_impl(
        &self,
        request: Request<SetupRequest>,
    ) -> Result<Response<CommonResponse>, Error> {
        let remote_address = request
            .remote_addr()
            .map_or_else(|| "Unknown".to_string(), |addr| addr.ip().to_string());

        let setup_request = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(setup_request.auth)?;

        let _ = self
            .context
            .datastore
            .device_setup(
                &jwt_token,
                token_info.account.device.id,
                setup_request.device_version,
                setup_request.device_uuid,
                remote_address,
            )
            .await?;

        Ok(Response::new(CommonResponse {
            message: String::from("Device setup completed successfully"),
        }))
    }
}
