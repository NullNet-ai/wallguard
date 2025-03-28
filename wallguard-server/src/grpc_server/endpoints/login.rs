use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tonic::{Request, Response};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{Authentication, LoginRequest},
};

impl WallGuardImpl {
    pub(crate) async fn login_impl(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<Authentication>, Error> {
        let login_request = request.into_inner();

        let token = self
            .context
            .datastore
            .login(login_request.app_id, login_request.app_secret)
            .await?;

        if token.is_empty() {
            return Err("Datastore login failed: Wrong credentials").handle_err(location!());
        }

        Ok(Response::new(Authentication { token }))
    }
}
