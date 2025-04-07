use crate::grpc_server::server::WallGuardImpl;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;

impl WallGuardImpl {
    pub(crate) fn authenticate(token: &str) -> Result<(String, Token), Error> {
        let token_info = Token::from_jwt(token).handle_err(location!())?;

        Ok((token.to_string(), token_info))
    }
}
