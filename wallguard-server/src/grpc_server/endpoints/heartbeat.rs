use nullnet_liberror::Error;
use tonic::{Request, Response};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{HeartbeatRequest, HeartbeatResponse},
};

impl WallGuardImpl {
    pub(crate) async fn heartbeat_impl(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Error> {
        let heartbeat_request = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(heartbeat_request.auth)?;

        let device_id = token_info.account.device.id;

        let is_remote_access_enabled = self
            .context
            .tunnel
            .lock()
            .await
            .get_profile_by_id(&device_id)
            .is_some();

        let device_info = self
            .context
            .datastore
            .heartbeat(&jwt_token, device_id)
            .await?;

        Ok(Response::new(HeartbeatResponse {
            status: device_info.status.into(),
            is_remote_access_enabled,
            is_monitoring_enabled: device_info.is_monitoring_enabled,
        }))
    }
}
