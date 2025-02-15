use axum::{http::StatusCode, Json};
use if_addrs::get_if_addrs;
use serde::Serialize;
use std::net::IpAddr;

#[derive(Serialize)]
pub(crate) struct AddrInfo {
    pub address: String,
    pub version: u8,
}

#[derive(Serialize)]
pub(crate) struct GetAddrResponse {
    pub values: Vec<AddrInfo>,
}

pub async fn get_addr() -> (StatusCode, Json<GetAddrResponse>) {
    let mut response = GetAddrResponse { values: vec![] };

    if let Ok(interfaces) = get_if_addrs() {
        for iface in interfaces {
            if iface.is_loopback() {
                continue;
            }

            match iface.ip() {
                IpAddr::V4(ipv4_addr) => response.values.push(AddrInfo {
                    address: ipv4_addr.to_string(),
                    version: 4,
                }),
                IpAddr::V6(ipv6_addr) => response.values.push(AddrInfo {
                    address: ipv6_addr.to_string(),
                    version: 6,
                }),
            };
        }
    }

    (StatusCode::OK, Json(response))
}
