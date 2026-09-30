use crate::datastore::Datastore;
use crate::orchestrator::Orchestrator;
use crate::reverse_tunnel::ReverseTunnel;
use crate::token_provider::TokenProvider;
use crate::tunneling::TunnelsManager;

use nullnet_liberror::Error;

// Unfortunately, we have to use both root and system device credentials because:
// - The system device cannot fetch data outside its own organization; only the root account can do that.
// - We cannot use the root account for everything because it cannot create records in the database.

pub static ROOT_ACCOUNT_ID: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("ROOT_ACCOUNT_ID").unwrap_or_else(|_| {
        log::warn!("'ROOT_ACCOUNT_ID' environment variable not set");
        String::new()
    })
});

pub static ROOT_ACCOUNT_SECRET: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("ROOT_ACCOUNT_SECRET").unwrap_or_else(|_| {
        log::warn!("'ROOT_ACCOUNT_SECRET' environment variable not set");
        String::new()
    })
});

pub static SYSTEM_ACCOUNT_ID: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("SYSTEM_ACCOUNT_ID").unwrap_or_else(|_| {
        log::warn!("'SYSTEM_ACCOUNT_ID' environment variable not set");
        String::new()
    })
});

pub static SYSTEM_ACCOUNT_SECRET: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("SYSTEM_ACCOUNT_SECRET").unwrap_or_else(|_| {
        log::warn!("'SYSTEM_ACCOUNT_SECRET' environment variable not set");
        String::new()
    })
});

#[derive(Debug, Clone)]
pub struct AppContext {
    pub datastore: Datastore,
    pub orchestractor: Orchestrator,
    pub tunnel: ReverseTunnel,

    pub root_token_provider: TokenProvider,
    pub sysdev_token_provider: TokenProvider,

    pub tunnels_manager: TunnelsManager,

    /// See [`store_telemetry_in_datastore`].
    pub store_telemetry: bool,
}

impl AppContext {
    pub async fn new() -> Result<Self, Error> {
        let datastore = Datastore::new().await?;
        let orchestractor = Orchestrator::new();
        let tunnel = ReverseTunnel::new();

        let sysdev_token_provider = TokenProvider::new(
            SYSTEM_ACCOUNT_ID.to_string(),
            SYSTEM_ACCOUNT_SECRET.to_string(),
            false,
            datastore.clone(),
        );

        let root_token_provider = TokenProvider::new(
            ROOT_ACCOUNT_ID.to_string(),
            ROOT_ACCOUNT_SECRET.to_string(),
            true,
            datastore.clone(),
        );

        let tunnels_manager = TunnelsManager::new();

        let store_telemetry = store_telemetry_in_datastore();
        if !store_telemetry {
            log::warn!(
                "STORE_TELEMETRY_IN_DATASTORE=false: connections, system resources and \
                 heartbeats will be accepted but not saved to the datastore"
            );
        }

        Ok(Self {
            datastore,
            orchestractor,
            tunnel,
            sysdev_token_provider,
            root_token_provider,
            tunnels_manager,
            store_telemetry,
        })
    }
}

/// Reads `STORE_TELEMETRY_IN_DATASTORE`. When `false`, connections, system
/// resources and heartbeats reported by agents are accepted but not written
/// to the datastore.
/// Defaults to `true`.
fn store_telemetry_in_datastore() -> bool {
    const NAME: &str = "STORE_TELEMETRY_IN_DATASTORE";

    let Ok(raw) = std::env::var(NAME) else {
        return true;
    };

    match raw.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => {
            log::warn!("{NAME} is set to {raw:?}, which is not a valid boolean; using true");
            true
        }
    }
}
