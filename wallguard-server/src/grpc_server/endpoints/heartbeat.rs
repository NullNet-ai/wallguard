use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{HeartbeatRequest, HeartbeatResponse},
    tunnel::RAType,
};
use nullnet_liberror::Error;
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn heartbeat_impl(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Error> {
        let heartbeat_request = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(heartbeat_request.auth)?;

        let device_id = token_info.account.device.id;

        let (remote_shell_enabled, remote_ui_enabled) = {
            let tunnel = self.context.tunnel.lock().await;

            let remote_shell_enabled = tunnel
                .get_profile_by_device_id(&device_id, &RAType::Shell)
                .await
                .is_some();

            let remote_ui_enabled = tunnel
                .get_profile_by_device_id(&device_id, &RAType::UI)
                .await
                .is_some();

            (remote_shell_enabled, remote_ui_enabled)
        };

        let device_info = self
            .context
            .datastore
            .heartbeat(&jwt_token, device_id)
            .await?;

        Ok(Response::new(HeartbeatResponse {
            status: device_info.status.into(),
            remote_shell_enabled,
            remote_ui_enabled,
            is_monitoring_enabled: device_info.is_monitoring_enabled,
        }))
    }
}
