use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Ssh,
    Http,
}

impl TryFrom<&str> for TunnelType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "ssh" => Ok(TunnelType::Ssh),
            "http" => Ok(TunnelType::Http),
            other => {
                Err(format!("Tunnel of type {other} is not supported")).handle_err(location!())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunnelModel {
    pub device_id: String,
    pub tunnel_type: TunnelType,
    pub service_id: String,
}
