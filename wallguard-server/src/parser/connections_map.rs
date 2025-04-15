use crate::parser::ip_header::IpHeader;
use crate::parser::parsed_message::{ParsedMessage, ParsedRecord};
use crate::parser::transport_header::TransportHeader;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

pub struct ConnectionsMap {
    pub connections: HashMap<ConnectionKey, ConnectionValue>,
}

impl ConnectionsMap {
    pub fn new() -> Self {
        ConnectionsMap {
            connections: HashMap::new(),
        }
    }

    pub fn into_parsed_message(self) -> ParsedMessage {
        let records = self
            .connections
            .into_iter()
            .map(|(key, value)| ParsedRecord {
                connection_key: key,
                connection_value: value,
            })
            .collect();
        ParsedMessage { records }
    }
}

#[derive(Debug, Serialize, Hash, Eq, PartialEq)]
pub struct ConnectionKey {
    pub device_id: String,
    pub interface_name: String,
    #[serde(flatten)]
    pub ip_header: IpHeader,
    #[serde(flatten)]
    pub transport_header: TransportHeader,
}

impl ConnectionKey {
    pub fn new(
        device_id: String,
        interface_name: String,
        ip_header: IpHeader,
        transport_header: TransportHeader,
    ) -> Self {
        ConnectionKey {
            device_id,
            interface_name,
            ip_header,
            transport_header,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionValue {
    pub timestamp: String,
    pub packets: usize,
    pub bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_ip: Option<IpAddr>,
}

impl ConnectionValue {
    pub fn new(timestamp: String, bytes: usize, remote_ip: Option<IpAddr>) -> Self {
        ConnectionValue {
            timestamp,
            packets: 1,
            bytes,
            remote_ip,
        }
    }

    pub fn update(&mut self, bytes: usize) {
        self.packets += 1;
        self.bytes += bytes;
    }
}
