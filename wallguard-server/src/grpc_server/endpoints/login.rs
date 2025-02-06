use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{Authentication, LoginRequest},
};

impl WallGuardImpl {
    pub(crate) async fn login_impl(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<Authentication>, Status> {
        let login_request = request.into_inner();

        let token = self
            .datastore
            .as_ref()
            .ok_or_else(|| Status::internal("Datastore is unavailable"))?
            .login(login_request.app_id, login_request.app_secret)
            .await
            .map_err(|e| Status::internal(format!("Datastore login failed: {e:?}")))?;

        if token.is_empty() {
            return Err(Status::internal(
                "Datastore login failed: Wrong credentials",
            ));
        }

        Ok(Response::new(Authentication { token }))
    }
}
