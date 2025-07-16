use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Alias {
    pub r#type: String,
    pub name: String,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rule {
    pub disabled: bool,
    pub r#type: String,
    pub policy: String,
    pub protocol: String,
    pub source_inversed: bool,
    pub source_port: String,
    pub source_addr: String,
    pub source_type: String,
    pub destination_port: String,
    pub destination_addr: String,
    pub destination_type: String,
    pub destination_inversed: bool,
    pub description: String,
    pub interface: String,
    pub order: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    pub device: String,
    pub addresses: Vec<IpAddress>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IpAddress {
    pub address: String,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SSHConfig {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Configuration {
    pub rules: Vec<Rule>,
    pub aliases: Vec<Alias>,
    pub interfaces: Vec<NetworkInterface>,
    pub raw_content: String,
    pub hostname: String,
    pub gui_protocol: String,
    pub ssh: SSHConfig,
}
