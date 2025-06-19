use nullnet_liberror::Error;

use crate::datastore::Datastore;
use crate::orchestrator::Orchestrator;
use crate::reverse_tunnel::ReverseTunnel;
use crate::token_provider::TokenProvider;

pub static SYSTEM_APP_ID: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("SYSTEM_APP_ID").unwrap_or_else(|_| {
        log::warn!("'SYSTEM_APP_ID' environment variable not set");
        String::new()
    })
});

pub static SYSTEM_APP_SECRET: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("SYSTEM_APP_SECRET").unwrap_or_else(|_| {
        log::warn!("'SYSTEM_APP_SECRET' environment variable not set");
        String::new()
    })
});

#[derive(Debug, Clone)]
pub struct AppContext {
    pub datastore: Datastore,
    pub orchestractor: Orchestrator,
    pub tunnel: ReverseTunnel,
    pub token_provider: TokenProvider,
}

impl AppContext {
    pub async fn new() -> Result<Self, Error> {
        let datastore = Datastore::new().await?;
        let orchestractor = Orchestrator::new();
        let tunnel = ReverseTunnel::new();

        let token_provider = TokenProvider::new(
            SYSTEM_APP_ID.to_string(),
            SYSTEM_APP_SECRET.to_string(),
            true,
            datastore.clone(),
        );

        Ok(Self {
            datastore,
            orchestractor,
            tunnel,
            token_provider,
        })
    }
}
