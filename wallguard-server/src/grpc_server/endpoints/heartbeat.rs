use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{HeartbeatRequest, HeartbeatResponse},
    utils::map_status_value_to_enum,
};

impl WallGuardImpl {
    pub(crate) async fn heartbeat_impl(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let heartbeat_request = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(heartbeat_request.auth)?;

        let (status, is_remote_access_enabled, is_monitoring_enabled) = self
            .datastore
            .as_ref()
            .ok_or_else(|| Status::internal("Datastore is unavailable"))?
            .heartbeat(&jwt_token, token_info.account.device.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(HeartbeatResponse {
            status: map_status_value_to_enum(&status).into(),
            is_remote_access_enabled,
            is_monitoring_enabled,
        }))
    }
}
