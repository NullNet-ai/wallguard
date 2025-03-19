use super::{
    ra_type::RAType,
    utils::{generate_addr, generate_uuid_str},
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
}

impl ClientProfile {
    pub async fn new(device_id: &str, ra_type: &str) -> Result<Self, Error> {
        let id = generate_uuid_str();
        let visitor_addr = generate_addr().await?;
        let ra_type = RAType::from_str(ra_type)?;

        Ok(Self {
            id,
            visitor_addr,
            ra_type,
            device_id: device_id.to_owned(),
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
}

impl Profile for ClientProfile {
    fn get_unique_id(&self) -> String {
        todo!()
    }

    fn get_visitor_addr(&self) -> SocketAddr {
        self.visitor_addr
    }
}
