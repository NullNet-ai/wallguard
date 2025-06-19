use nullnet_liberror::{Error, ErrorHandler, Location, location};

use crate::datastore::Datastore;
use crate::datastore::builders::LoginRequestBuilder;

impl Datastore {
    /// Performs login using the provided `app_id` and `app_secret`.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The application/client identifier.
    /// * `app_secret` - The secret or password associated with the `app_id`.
    /// * `is_root` - Whether the credentials belong to a ROOT account
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the authentication token if successful.
    /// * `Err(Error)` if the request fails or the server returns an error.
    pub async fn login(
        &self,
        app_id: &str,
        app_secret: &str,
        is_root: bool,
    ) -> Result<String, Error> {
        let request = LoginRequestBuilder::new()
            .account_id(app_id)
            .account_secret(app_secret)
            .set_root(is_root)
            .build();

        let response = self.inner.clone().login(request).await?;

        validate_token(&response.token)?;

        Ok(response.token)
    }
}

/// Validates that the provided token is non-empty.
fn validate_token(token: &str) -> Result<(), Error> {
    match token.is_empty() {
        true => Err("Unauthenticated: wrong app_id and/or app_secret").handle_err(location!()),
        false => Ok(()),
    }
}
