use crate::{
    app_context::AppContext,
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{CommonResponse, ConfigSnapshot, ConfigStatus},
    tunnel::RAType,
};
use libfireparse::{Configuration, FileData, FireparseError, Parser, Platform};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn handle_config_impl(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<CommonResponse>, Error> {
        let snapshot = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(&snapshot.token)?;

        let snapshot_mapped = snapshot
            .files
            .into_iter()
            .map(|sf| FileData {
                filename: sf.filename,
                content: sf.contents,
            })
            .collect();

        let configuration = Parser::parse(Platform::PfSense, snapshot_mapped)
            .map_err(|e| match e {
                FireparseError::UnsupportedPlatform(message)
                | FireparseError::ParserError(message) => message,
            })
            .handle_err(location!())?;

        Self::internal_handle_gui_protocol_change(
            &configuration,
            &self.context,
            &jwt_token,
            &token_info,
        )
        .await;

        let config_id = &self
            .context
            .datastore
            .config_upload(
                &jwt_token,
                token_info.account.device.id,
                configuration,
                ConfigStatus::try_from(snapshot.status).unwrap_or(ConfigStatus::CsUndefined),
            )
            .await?;

        Ok(Response::new(CommonResponse {
            message: format!("Successfully created\\updated a configuration, id '{config_id}'"),
        }))
    }

    // Terminate UI session if exists
    async fn internal_handle_gui_protocol_change(
        config: &Configuration,
        context: &AppContext,
        token: &str,
        info: &Token,
    ) {
        match context
            .tunnel
            .lock()
            .await
            .get_profile_if_online_by_device_id(&info.account.device.id, &RAType::UI)
            .await
        {
            Some(profile) => {
                if profile.ui_proto() == config.gui_protocol {
                    return;
                }
            }
            None => {
                return;
            }
        };

        let _ = context
            .tunnel
            .lock()
            .await
            .remove_profile(&info.account.device.id, &RAType::UI)
            .await;

        let _ = context
            .clients_manager
            .lock()
            .await
            .force_heartbeat(&info.account.device.id)
            .await;

        let _ = context
            .datastore
            .device_terminate_remote_session(token, info.account.device.id.clone(), RAType::UI)
            .await;
    }
}
