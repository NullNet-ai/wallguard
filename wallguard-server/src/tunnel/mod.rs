mod config;
mod profile_ex;
mod ra_type;
mod utils;

use config::Config;
use nullnet_liberror::Error;
use nullnet_libtunnel::Server;
pub use profile_ex::ProfileEx;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TunnelServer {
    inner: Arc<Server>,
    profiles: HashMap<String, ProfileEx>,
}

impl TunnelServer {
    pub fn new() -> Self {
        let config = Config::from_env();
        let inner = Server::new(config.addr, config.heartbeat_interval);
        let profiles = HashMap::new();
        Self {
            inner: Arc::new(inner),
            profiles,
        }
    }

    pub async fn add_profile(&mut self, profile: ProfileEx) -> Result<(), Error> {
        self.profiles.insert(profile.device_id(), profile.clone());
        self.inner.register_profile(profile.into()).await
    }

    pub async fn remove_profile(&mut self, device_id: &str) -> Result<(), Error> {
        self.profiles.remove(device_id);
        self.inner.remove_profile(device_id).await
    }

    pub fn run(&self) {
        let inner = self.inner.clone();
        let _ = tokio::spawn(async move {
            if let Err(err) = inner.run().await {
                panic!("Tunnel server crashed with error: {}", err.to_str());
            }
        });
    }
}
