use device_uuid::retrieve_device_uuid;
use nullnet_liberror::{ErrorHandler, Location, location};
pub use platform::Platform;
pub use target_os::TargetOs;

mod device_uuid;
mod platform;
mod target_os;

#[derive(Debug, Clone)]
pub struct ClientData {
    pub(crate) target_os: TargetOs,
    pub(crate) platform: Platform,
    pub(crate) uuid: String,
    pub(crate) category: String,
}

impl TryFrom<String> for ClientData {
    type Error = nullnet_liberror::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let platform = Platform::try_from(value)?;

        let uuid = retrieve_device_uuid()
            .ok_or("Failed to retrieve device UUID")
            .handle_err(location!())?;

        let target_os = TargetOs::new();

        let category = String::from("Firewall");

        Ok(Self {
            target_os,
            platform,
            uuid,
            category,
        })
    }
}
