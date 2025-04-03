use super::{
    ra_type::RAType,
    utils::{generate_addr, generate_random_token, generate_uuid_str},
};
use nullnet_liberror::Error;
use nullnet_libtunnel::Profile;
use std::{net::SocketAddr, str::FromStr};

#[derive(Clone, Debug)]
pub struct ClientProfile {
    id: String,
    visitor_addr: SocketAddr,
    device_id: String,
    ra_type: RAType,
    visitor_token: String,
    public_session_id: String,
    ui_proto: String,
}

impl ClientProfile {
    pub async fn new(device_id: &str, ra_type: &str, ui_proto: String) -> Result<Self, Error> {
        let id = generate_uuid_str();
        let visitor_addr = generate_addr().await?;
        let ra_type = RAType::from_str(ra_type)?;
        let visitor_token = generate_random_token(128);
        let public_session_id = generate_random_token(32);

        Ok(Self {
            id,
            visitor_addr,
            ra_type,
            device_id: device_id.to_owned(),
            visitor_token,
            public_session_id,
            ui_proto,
        })
    }

    pub fn tunnel_id(&self) -> String {
        self.id.clone()
    }

    pub fn device_id(&self) -> String {
        self.device_id.clone()
    }

    pub fn remote_access_type(&self) -> RAType {
        self.ra_type
    }

    pub fn public_session_id(&self) -> String {
        self.public_session_id.clone()
    }

    pub fn ui_proto(&self) -> &str {
        &self.ui_proto
    }
}

impl Profile for ClientProfile {
    fn get_unique_id(&self) -> String {
        self.id.clone()
    }

    fn get_visitor_addr(&self) -> SocketAddr {
        self.visitor_addr
    }

    fn get_visitor_token(&self) -> Option<String> {
        Some(self.visitor_token.clone())
    }
}
