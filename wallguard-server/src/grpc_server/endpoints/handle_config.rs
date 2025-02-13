use libfireparse::{FileData, FireparseError, Parser, Platform};
use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{CommonResponse, ConfigSnapshot, ConfigStatus},
};

impl WallGuardImpl {
    pub(crate) async fn handle_config_impl(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<CommonResponse>, Status> {
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

        let configuration =
            Parser::parse(Platform::PfSense, snapshot_mapped).map_err(|e| match e {
                FireparseError::UnsupportedPlatform(msg) | FireparseError::ParserError(msg) => {
                    Status::internal(msg)
                }
            })?;

        let created_id = &self
            .datastore
            .config_upload(
                &jwt_token,
                token_info.account.device.id,
                configuration,
                convert_status(
                    ConfigStatus::try_from(snapshot.status).unwrap_or(ConfigStatus::CsUndefined),
                ),
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CommonResponse {
            success: true,
            message: format!("Configuration created [ID '{created_id}']"),
        }))
    }
}

fn convert_status(status: ConfigStatus) -> String {
    match status {
        ConfigStatus::CsDraft => String::from("Draft"),
        ConfigStatus::CsApplied => String::from("Applied"),
        ConfigStatus::CsUndefined => String::from("Undefined"),
    }
}
