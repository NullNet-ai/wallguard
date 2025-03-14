use super::{
    ra_type::RAType,
    utils::{generate_addr, generate_uuid_str},
};
use nullnet_liberror::Error;
use nullnet_libtunnel::ClientProfile;
use std::str::FromStr;

#[derive(Clone)]
pub struct ProfileEx {
    profile: ClientProfile,
    device_id: String,
    ra_type: RAType,
}

impl ProfileEx {
    pub async fn new(device_id: &str, ra_type: &str) -> Result<Self, Error> {
        let id = generate_uuid_str();
        let visitor_addr = generate_addr().await?;
        let ra_type = RAType::from_str(ra_type)?;

        Ok(Self {
            device_id: device_id.to_owned(),
            profile: ClientProfile { id, visitor_addr },
            ra_type,
        })
    }

    pub fn tunnel_id(&self) -> String {
        self.profile.id.clone()
    }

    pub fn device_id(&self) -> String {
        self.device_id.clone()
    }

    pub fn visitor_port(&self) -> u16 {
        self.profile.visitor_addr.port()
    }

    pub fn remote_access_type(&self) -> RAType {
        self.ra_type
    }
}

impl From<ProfileEx> for ClientProfile {
    fn from(value: ProfileEx) -> Self {
        value.profile
    }
}
