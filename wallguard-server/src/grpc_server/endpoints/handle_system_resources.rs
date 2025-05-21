use crate::proto::wallguard::SystemResources;
use crate::{grpc_server::server::WallGuardImpl, proto::wallguard::CommonResponse};
use nullnet_liberror::Error;
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn handle_system_resources_impl(
        &self,
        request: Request<SystemResources>,
    ) -> Result<Response<CommonResponse>, Error> {
        let resources = request.into_inner();
        let (jwt_token, _) = Self::authenticate(&resources.token)?;

        // todo ???
        // self.experimental_create_system_resources(...).await;

        let _ = self
            .context
            .datastore
            .system_resources_insert(&jwt_token, resources)
            .await?;

        Ok(Response::new(CommonResponse {
            message: String::from("System resources successfully inserted"),
        }))
    }
}
