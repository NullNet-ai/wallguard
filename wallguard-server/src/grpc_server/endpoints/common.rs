use nullnet_libtoken::Token;
use tonic::Status;

use crate::{grpc_server::server::WallGuardImpl, proto::wallguard::Authentication};

impl WallGuardImpl {
    // @TODO: Optimize? Looks like there is an unnecessary copy
    pub(crate) fn authenticate(auth: Option<Authentication>) -> Result<(String, Token), Status> {
        let Some(auth_message) = auth else {
            return Err(Status::internal("Authentication token is missing"));
        };

        let jwt_token = auth_message.token.clone();

        let token_info =
            Token::from_jwt(&jwt_token).map_err(|e| Status::internal(e.to_string()))?;

        Ok((jwt_token, token_info))
    }
}
