use crate::datastore::Datastore;
use data::AuthData;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;
use std::sync::Arc;
use tokio::sync::Mutex;

mod data;

/// `TokenProvider` is responsible for managing and refreshing an authentication token (JWT)
/// as needed. It ensures that any consumer always receives a valid token when calling `get()`.
#[derive(Debug, Clone)]
pub struct TokenProvider {
    datastore: Datastore,
    data: Arc<Mutex<AuthData>>,
}

impl TokenProvider {
    /// Creates a new `TokenProvider` instance.
    ///
    /// # Arguments
    /// * `app_id` - Application identifier used for authentication.
    /// * `app_secret` - Secret associated with the application.
    /// * `datastore` - A `Datastore` instance used to perform login requests.
    pub fn new(app_id: &str, app_secret: &str, datastore: Datastore) -> Self {
        let data = AuthData::new(app_id, app_secret);
        Self {
            datastore,
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Returns a valid JWT token.
    ///
    /// If the currently stored token is missing or expired, it performs a login
    /// using the stored credentials and updates the token.
    ///
    /// # Returns
    /// * `Ok(String)` - The valid JWT token.
    /// * `Err(Error)` - If login or token parsing fails.
    pub async fn get(&self) -> Result<String, Error> {
        let mut lock = self.data.lock().await;

        if lock.needs_refresh() {
            let jwt = self.datastore.login(&lock.app_id, &lock.app_secret).await?;

            let token = Token::from_jwt(&jwt).handle_err(location!())?;

            lock.token = Some(token);
        }

        Ok(lock.token.as_ref().unwrap().jwt.clone())
    }
}
