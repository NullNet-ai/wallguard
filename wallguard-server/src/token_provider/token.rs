use super::models::Account;
use base64::Engine as _;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPIRATION_MARGIN: u64 = 60 * 5;

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub account: Account,
    pub signed_in_account: Account,
    pub iat: u64,
    pub exp: u64,
    #[serde(skip)]
    pub jwt: String,
}

impl Token {
    /// Decodes a JWT and parses its payload into a `Token` struct.
    ///
    /// # Arguments
    /// * `jwt` - A JWT string consisting of three parts separated by periods (`.`).
    ///
    /// # Returns
    /// * `Ok(Token)` if the token is successfully decoded and parsed.
    /// * `Err(Error)` if the token is malformed, Base64 decoding fails, or payload deserialization fails.
    #[allow(clippy::missing_errors_doc)]
    pub fn from_jwt(jwt: impl Into<String>) -> Result<Self, Error> {
        let jwt = jwt.into();

        let parts: Vec<&str> = jwt.split('.').collect();

        if parts.len() != 3 {
            return Err("Malformed JWT").handle_err(location!());
        }

        let decoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .handle_err(location!())?;

        let mut token: Token = serde_json::from_slice(&decoded_payload).handle_err(location!())?;

        token.jwt = jwt;

        Ok(token)
    }

    /// Checks if the token has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        // consider the token expired if duration_since fails
        let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return true;
        };
        self.exp <= (duration.as_secs() - EXPIRATION_MARGIN)
    }
}
