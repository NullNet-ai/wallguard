use crate::datastore::Datastore;
use data::AuthData;
use nullnet_liberror::Error;
use std::sync::Arc;
pub use token::Token;
use tokio::sync::Mutex;

mod data;
mod models;
mod token;

#[derive(Debug, Clone)]
pub struct TokenProvider {
    datastore: Datastore,
    data: Arc<Mutex<AuthData>>,
}

impl TokenProvider {
    pub fn new(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        datastore: Datastore,
    ) -> Self {
        let data = AuthData::new(app_id, app_secret);
        Self {
            datastore,
            data: Arc::new(Mutex::new(data)),
        }
    }

    pub async fn get(&self) -> Result<Arc<Token>, Error> {
        let mut lock = self.data.lock().await;

        if lock.needs_refresh() {
            let jwt = self.datastore.login(&lock.app_id, &lock.app_secret).await?;

            let token = Token::from_jwt(&jwt)?;

            lock.token = Some(Arc::new(token));
        }

        Ok(lock.token.as_ref().unwrap().clone())
    }
}
