use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{ControlChannelRequest, ControlChannelResponse},
    tunnel::RAType,
};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn request_control_channel_impl(
        &self,
        request: Request<ControlChannelRequest>,
    ) -> Result<Response<ControlChannelResponse>, Error> {
        let control_channel_request = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(control_channel_request.auth)?;

        let device_id = token_info.account.device.id;

        let tunnel_lock = self.context.tunnel.lock().await;

        let Some(profile) = tunnel_lock.get_profile_by_device_id(&device_id).await else {
            return Err(format!(
                "No active tunnels opened for device {}",
                &device_id
            ))
            .handle_err(location!());
        };

        let tunnel_id = profile.tunnel_id();
        let ra_type = profile.remote_access_type();
        drop(tunnel_lock);

        let protocol = match ra_type {
            RAType::Shell => None,
            RAType::UI => {
                let protocol = self
                    .context
                    .datastore
                    .device_fetch_webgui_protocol(device_id, &jwt_token)
                    .await?;
                Some(protocol)
            }
        };

        Ok(Response::new(ControlChannelResponse {
            id: tunnel_id,
            r#type: ra_type.to_string(),
            protocol,
        }))
    }
}
