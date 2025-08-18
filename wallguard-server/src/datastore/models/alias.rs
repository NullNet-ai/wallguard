use serde::{Deserialize, Serialize};
use wallguard_common::protobuf::wallguard_models::Alias;

#[derive(Serialize, Deserialize, Clone)]
pub struct AliasModel {
    pub device_configuration_id: String,
    pub r#type: String,
    pub name: String,
    pub description: String,
    pub alias_status: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IpAliasModel {
    pub alias_id: String,
    pub ip: String,
    pub prefix: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PortAliasModel {
    pub alias_id: String,
    pub lower_port: i32,
    pub upper_port: i32,
}

// @TODO: Move to client's parser ??
impl AliasModel {
    pub fn extract_ip_aliases(&self, alias_from: &Alias, alias_id: &str) -> Vec<IpAliasModel> {
        let values: Vec<String> = alias_from.value.split(",").map(str::to_string).collect();

        if self.r#type == "host" {
            values
                .into_iter()
                .map(|value| IpAliasModel {
                    alias_id: alias_id.to_string(),
                    ip: value,
                    prefix: 32,
                })
                .collect()
        } else if self.r#type == "network" {
            values
                .into_iter()
                .map(|value| {
                    let split: Vec<&str> = value.split('/').collect();

                    let addr = split.first().unwrap_or(&"").to_string();

                    let prefix: i32 = split
                        .get(1)
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(0);

                    IpAliasModel {
                        alias_id: alias_id.to_string(),
                        ip: addr.clone(),
                        prefix,
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn extract_port_aliases(&self, alias_from: &Alias, alias_id: &str) -> Vec<PortAliasModel> {
        let values: Vec<String> = alias_from.value.split(",").map(str::to_string).collect();

        if self.r#type == "port" {
            values
                .into_iter()
                .map(|value| {
                    let split: Vec<&str> = value.split(':').collect();

                    let lower_port = split
                        .first()
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(0);

                    let upper_port = split
                        .get(1)
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(lower_port);

                    PortAliasModel {
                        alias_id: alias_id.to_string(),
                        lower_port,
                        upper_port,
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
}
