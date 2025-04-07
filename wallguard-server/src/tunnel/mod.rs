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
    devices_map: HashMap<RAType, HashMap<String, ClientProfile>>,
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

    pub async fn insert_profile(
        &mut self,
        profile: ClientProfile,
        ra_type: RAType,
    ) -> Result<(), Error> {
        self.devices_map
            .entry(ra_type)
            .or_default()
            .insert(profile.device_id(), profile.clone());

        self.sessions_map
            .insert(profile.public_session_id(), profile.clone());

        self.inner.insert_profile(profile).await
    }

    pub async fn remove_profile(&mut self, device_id: &str, ra_type: &RAType) -> Result<(), Error> {
        let devices_map = self
            .devices_map
            .get_mut(ra_type)
            .ok_or(format!("No open sessions with type {}", ra_type))
            .handle_err(location!())?;

        match devices_map.remove(device_id) {
            Some(profile) => {
                self.sessions_map.remove(&profile.public_session_id());
                self.inner.remove_profile(&profile.get_unique_id()).await
            }
            None => Err(format!("Device {} not found", device_id)).handle_err(location!()),
        }
    }

    pub async fn get_profile_by_device_id(
        &self,
        id: &str,
        ra_type: &RAType,
    ) -> Option<&ClientProfile> {
        self.devices_map
            .get(ra_type)
            .map(|map| map.get(id))
            .unwrap_or(None)
    }

    pub async fn get_profile_if_online_by_device_id(
        &self,
        device_id: &str,
        ra_type: &RAType,
    ) -> Option<&ClientProfile> {
        match self.get_profile_by_device_id(device_id, ra_type).await {
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

    pub fn does_profile_exist(&self, device_id: &str, ra_type: &RAType) -> bool {
        self.devices_map
            .get(ra_type)
            .map(|map| map.contains_key(device_id))
            .unwrap_or(false)
    }
}
