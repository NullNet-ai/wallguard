use serde::{Deserialize, Serialize};
use wallguard_common::protobuf::wallguard_service::{
    ServiceInfo as ServiceInfoGrpc, ServiceProtocol,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceInfo {
    pub device_id: String,
    pub address: String,
    pub port: u16,
    pub protocol: String,
}

impl ServiceInfo {
    pub fn new(data: ServiceInfoGrpc, device_id: String) -> Self {
        let proto = ServiceProtocol::try_from(data.protocol);
        Self {
            device_id,
            address: data.address,
            port: data.port as u16,
            protocol: match proto {
                Ok(ServiceProtocol::Http) => "http".into(),
                Ok(ServiceProtocol::Https) => "https".into(),
                _ => "unknown".into(),
            },
        }
    }
}
