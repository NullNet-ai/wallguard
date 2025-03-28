use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{CommonResponse, ConfigSnapshot, ConfigStatus},
};
use libfireparse::{FileData, FireparseError, Parser, Platform};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn handle_config_impl(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<CommonResponse>, Error> {
        let snapshot = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(snapshot.auth)?;

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
}
