use crate::proto::wallguard::Logs;
use crate::{grpc_server::server::WallGuardImpl, proto::wallguard::CommonResponse};
use nullnet_liberror::Error;
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn handle_logs_impl(
        &self,
        request: Request<Logs>,
    ) -> Result<Response<CommonResponse>, Error> {
        let logs = request.into_inner();
        let (jwt_token, _) = Self::authenticate(&logs.token)?;

        let _ = self
            .context
            .datastore
            .logs_insert(&jwt_token, logs.logs)
            .await?;

        Ok(Response::new(CommonResponse {
            message: String::from("Logs successfully inserted"),
        }))
    }
}
