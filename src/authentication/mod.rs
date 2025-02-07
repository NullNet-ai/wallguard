mod request_impl;
mod token_wrapper;

use request_impl::request_impl;
use std::sync::Arc;
use token_wrapper::TokenWrapper;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct AuthHandler {
    app_id: String,
    app_secret: String,
    server_addr: String,
    server_port: u16,
    token: Arc<Mutex<Option<TokenWrapper>>>,
}

impl AuthHandler {
    pub fn new(app_id: String, app_secret: String, server_addr: String, server_port: u16) -> Self {
        Self {
            app_id,
            app_secret,
            server_addr,
            server_port,
            token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn obtain_token_safe(&self) -> Result<String, String> {
        let mut token = self.token.lock().await;

        if token.as_ref().map_or(true, TokenWrapper::is_expired) {
            let new_token = request_impl(
                &self.server_addr,
                self.server_port,
                self.app_id.clone(),
                self.app_secret.clone(),
            )
            .await?;

            *token = Some(new_token)
        }

        Ok(token.as_ref().unwrap().jwt.clone())
    }
}
