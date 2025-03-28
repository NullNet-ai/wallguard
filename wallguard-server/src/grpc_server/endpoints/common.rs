use crate::{grpc_server::server::WallGuardImpl, proto::wallguard::Authentication};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;

impl WallGuardImpl {
    pub(crate) fn authenticate(auth: Option<Authentication>) -> Result<(String, Token), Error> {
        let Some(auth_message) = auth else {
            return Err("Authentication token is missing").handle_err(location!());
        };

        let jwt_token = auth_message.token.clone();

        let token_info = Token::from_jwt(&jwt_token).handle_err(location!())?;

        Ok((jwt_token, token_info))
    }
}
