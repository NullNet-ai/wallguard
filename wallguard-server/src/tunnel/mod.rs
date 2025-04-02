mod client_profile;
mod config;
mod monitor;
mod ra_type;
mod utils;

pub use client_profile::ClientProfile;
pub use config::Config;
pub use monitor::monitor_idle_profiles;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtunnel::{Profile, Server};
pub use ra_type::RAType;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TunnelServer {
    inner: Server<ClientProfile>,
    devices_map: HashMap<String, ClientProfile>,
    sessions_map: HashMap<String, ClientProfile>,
}

// @TODO: ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// Unnessesary copies are being created

impl TunnelServer {
    pub fn new() -> Self {
        let config = Config::from_env();
        let server = Server::new(config.into());

        Self {
            inner: server,
            devices_map: HashMap::new(),
            sessions_map: HashMap::new(),
        }
    }

    pub async fn insert_profile(&mut self, profile: ClientProfile) -> Result<(), Error> {
        self.devices_map
            .insert(profile.device_id(), profile.clone());
        self.sessions_map
            .insert(profile.public_session_id(), profile.clone());
        self.inner.insert_profile(profile).await
    }

    pub async fn remove_profile(&mut self, device_id: &str) -> Result<(), Error> {
        match self.devices_map.remove(device_id) {
            Some(profile) => {
                self.sessions_map.remove(&profile.public_session_id());
                self.inner.remove_profile(&profile.get_unique_id()).await
            }
            None => Err(format!("Device {} not found", device_id)).handle_err(location!()),
        }
    }

    pub async fn get_profile_by_device_id(&self, id: &str) -> Option<&ClientProfile> {
        self.devices_map.get(id)
    }

    pub async fn get_profile_if_online_by_device_id(
        &self,
        device_id: &str,
    ) -> Option<&ClientProfile> {
        match self.get_profile_by_device_id(device_id).await {
            Some(profile) => match self.inner.is_profile_online(&profile.get_unique_id()).await {
                true => Some(profile),
                false => None,
            },
            None => None,
        }
    }

    pub async fn get_profile_if_online_by_public_session_id(
        &self,
        session: &str,
    ) -> Option<&ClientProfile> {
        match self.sessions_map.get(session) {
            Some(profile) => match self.inner.is_profile_online(&profile.get_unique_id()).await {
                true => Some(profile),
                false => None,
            },
            None => None,
        }
    }

    pub fn does_profile_exist(&self, device_id: &str) -> bool {
        self.devices_map.contains_key(device_id)
    }
}
