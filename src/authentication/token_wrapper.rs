use crate::utils;
use nullnet_libtoken::Token;

const EXPIRATION_MARGIN: u64 = 60 * 5;

#[derive(Debug)]
pub struct TokenWrapper {
    pub jwt: String,
    pub info: Token,
}

impl TokenWrapper {
    pub fn from_jwt(jwt: String) -> Result<Self, String> {
        let info = Token::from_jwt(&jwt)?;
        Ok(Self { jwt, info })
    }

    pub fn is_expired(&self) -> bool {
        self.info.exp <= (utils::timestamp() - EXPIRATION_MARGIN)
    }
}
