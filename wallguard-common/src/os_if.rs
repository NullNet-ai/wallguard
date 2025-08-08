use super::serde_ext::*;
use bincode::{Error, deserialize, serialize};
use get_if_addrs::{IfAddr, get_if_addrs};
use pnet::datalink;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InterfaceSnapshot {
    pub name: String,
    pub is_up: bool,
    pub is_loopback: bool,
    pub is_multicast: bool,
    pub is_broadcast: bool,
    pub mac_address: Option<String>,
    pub interface_index: Option<u32>,
    #[serde(with = "serde_ipaddr_vec")]
    pub ip_addresses: Vec<IpAddr>,
    #[serde(with = "serde_ipaddr_option")]
    pub subnet_mask: Option<IpAddr>,
    #[serde(with = "serde_ipaddr_option")]
    pub gateway: Option<IpAddr>,
}

impl InterfaceSnapshot {
    pub fn serialize_snapshot(snapshot: &Vec<InterfaceSnapshot>) -> Result<Vec<u8>, Error> {
        serialize(snapshot)
    }

    pub fn deserialize_snapshot(data: &[u8]) -> Result<Vec<InterfaceSnapshot>, Error> {
        deserialize(data)
    }

    pub fn take_all() -> Vec<InterfaceSnapshot> {
        let interfaces = datalink::interfaces();
        let mut iface_map: HashMap<String, InterfaceSnapshot> = HashMap::new();

        for iface in interfaces {
            iface_map.insert(
                iface.name.clone(),
                InterfaceSnapshot {
                    name: iface.name.clone(),
                    is_up: iface.is_up(),
                    is_loopback: iface.is_loopback(),
                    is_multicast: iface.is_multicast(),
                    is_broadcast: iface.is_broadcast(),
                    mac_address: iface.mac.as_ref().map(|mac| mac.to_string()),
                    interface_index: Some(iface.index),
                    ip_addresses: Vec::new(),
                    subnet_mask: None,
                    gateway: None,
                },
            );
        }

        if let Ok(if_addrs) = get_if_addrs() {
            for iface in if_addrs {
                if let Some(entry) = iface_map.get_mut(&iface.name) {
                    match iface.addr {
                        IfAddr::V4(ipv4) => {
                            entry.ip_addresses.push(IpAddr::V4(ipv4.ip));
                            entry.subnet_mask = Some(IpAddr::V4(ipv4.netmask));
                        }
                        IfAddr::V6(ipv6) => {
                            entry.ip_addresses.push(IpAddr::V6(ipv6.ip));
                        }
                    }
                }
            }
        }

        iface_map.into_values().collect()
    }
}
