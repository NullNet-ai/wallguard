use super::wallguard_models::{FilterRule, NatRule};
use crate::protobuf::wallguard_models::{AddrInfo, PortInfo};
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};

impl Serialize for FilterRule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FilterRule", 25)?;

        state.serialize_field("disabled", &self.disabled)?;
        state.serialize_field("policy", &self.policy)?;
        state.serialize_field("protocol", &self.protocol)?;
        state.serialize_field("source_inversed", &self.source_inversed)?;

        if let Some(sp) = &self.source_port {
            state.serialize_field("source_port_value", &sp.value)?;
            state.serialize_field("source_port_operator", &sp.operator)?;
        }

        if let Some(sa) = &self.source_addr {
            state.serialize_field("source_ip_value", &sa.value)?;
            state.serialize_field("source_ip_operator", &sa.operator)?;
            state.serialize_field("source_ip_version", &sa.version)?;
        }

        state.serialize_field("source_type", &self.source_type)?;
        state.serialize_field("destination_inversed", &self.destination_inversed)?;

        if let Some(dp) = &self.destination_port {
            state.serialize_field("destination_port_value", &dp.value)?;
            state.serialize_field("destination_port_operator", &dp.operator)?;
        }

        if let Some(da) = &self.destination_addr {
            state.serialize_field("destination_ip_value", &da.value)?;
            state.serialize_field("destination_ip_operator", &da.operator)?;
            state.serialize_field("destination_ip_version", &da.version)?;
        }

        state.serialize_field("destination_type", &self.destination_type)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("interface", &self.interface)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("order", &self.order)?;
        state.serialize_field("associated_rule_id", &self.associated_rule_id)?;

        state.serialize_field("table", &self.table)?;
        state.serialize_field("chain", &self.chain)?;
        state.serialize_field("family", &self.family)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for FilterRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawFilterRule {
            disabled: bool,
            policy: String,
            protocol: String,
            source_inversed: bool,
            source_port_value: Option<String>,
            source_port_operator: Option<String>,
            source_ip_value: Option<String>,
            source_ip_operator: Option<String>,
            source_ip_version: Option<i32>,
            source_type: String,
            destination_inversed: bool,
            destination_port_value: Option<String>,
            destination_port_operator: Option<String>,
            destination_ip_value: Option<String>,
            destination_ip_operator: Option<String>,
            destination_ip_version: Option<i32>,
            destination_type: String,
            description: String,
            interface: String,
            id: u32,
            order: u32,
            associated_rule_id: String,
            table: Option<String>,
            chain: Option<String>,
            family: Option<String>,
        }

        let raw = RawFilterRule::deserialize(deserializer)?;

        let source_port = match (raw.source_port_value, raw.source_port_operator) {
            (Some(value), Some(operator)) => Some(PortInfo { value, operator }),
            _ => None,
        };

        let source_addr = match (
            raw.source_ip_value,
            raw.source_ip_operator,
            raw.source_ip_version,
        ) {
            (Some(value), Some(operator), Some(version)) => Some(AddrInfo {
                value,
                operator,
                version,
            }),
            _ => None,
        };

        let destination_port = match (raw.destination_port_value, raw.destination_port_operator) {
            (Some(value), Some(operator)) => Some(PortInfo { value, operator }),
            _ => None,
        };

        let destination_addr = match (
            raw.destination_ip_value,
            raw.destination_ip_operator,
            raw.destination_ip_version,
        ) {
            (Some(value), Some(operator), Some(version)) => Some(AddrInfo {
                value,
                operator,
                version,
            }),
            _ => None,
        };

        Ok(FilterRule {
            disabled: raw.disabled,
            policy: raw.policy,
            protocol: raw.protocol,
            source_inversed: raw.source_inversed,
            source_port,
            source_addr,
            source_type: raw.source_type,
            destination_inversed: raw.destination_inversed,
            destination_port,
            destination_addr,
            destination_type: raw.destination_type,
            description: raw.description,
            interface: raw.interface,
            id: raw.id,
            order: raw.order,
            associated_rule_id: raw.associated_rule_id,
            table: raw.table.unwrap_or_default(),
            chain: raw.chain.unwrap_or_default(),
            family: raw.family.unwrap_or_default(),
        })
    }
}

impl Serialize for NatRule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NatRule", 25)?;

        state.serialize_field("disabled", &self.disabled)?;
        state.serialize_field("protocol", &self.protocol)?;
        state.serialize_field("source_inversed", &self.source_inversed)?;

        if let Some(sp) = &self.source_port {
            state.serialize_field("source_port_value", &sp.value)?;
            state.serialize_field("source_port_operator", &sp.operator)?;
        }

        if let Some(sa) = &self.source_addr {
            state.serialize_field("source_ip_value", &sa.value)?;
            state.serialize_field("source_ip_operator", &sa.operator)?;
            state.serialize_field("source_ip_version", &sa.version)?;
        }

        state.serialize_field("source_type", &self.source_type)?;
        state.serialize_field("destination_inversed", &self.destination_inversed)?;

        if let Some(dp) = &self.destination_port {
            state.serialize_field("destination_port_value", &dp.value)?;
            state.serialize_field("destination_port_operator", &dp.operator)?;
        }

        if let Some(da) = &self.destination_addr {
            state.serialize_field("destination_ip_value", &da.value)?;
            state.serialize_field("destination_ip_operator", &da.operator)?;
            state.serialize_field("destination_ip_version", &da.version)?;
        }

        state.serialize_field("destination_type", &self.destination_type)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("interface", &self.interface)?;
        state.serialize_field("redirect_ip", &self.redirect_ip)?;
        state.serialize_field("redirect_port", &self.redirect_port)?;
        state.serialize_field("order", &self.order)?;
        state.serialize_field("associated_rule_id", &self.associated_rule_id)?;

        state.serialize_field("table", &self.table)?;
        state.serialize_field("chain", &self.chain)?;
        state.serialize_field("family", &self.family)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for NatRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawNatRule {
            disabled: bool,
            protocol: String,
            source_inversed: bool,
            source_port_value: Option<String>,
            source_port_operator: Option<String>,
            source_ip_value: Option<String>,
            source_ip_operator: Option<String>,
            source_ip_version: Option<i32>,
            source_type: String,
            destination_inversed: bool,
            destination_port_value: Option<String>,
            destination_port_operator: Option<String>,
            destination_ip_value: Option<String>,
            destination_ip_operator: Option<String>,
            destination_ip_version: Option<i32>,
            destination_type: String,
            description: String,
            interface: String,
            redirect_ip: String,
            redirect_port: u32,
            order: u32,
            associated_rule_id: String,
            table: Option<String>,
            chain: Option<String>,
            family: Option<String>,
        }

        let raw = RawNatRule::deserialize(deserializer)?;

        let source_port = match (raw.source_port_value, raw.source_port_operator) {
            (Some(value), Some(operator)) => Some(PortInfo { value, operator }),
            _ => None,
        };

        let source_addr = match (
            raw.source_ip_value,
            raw.source_ip_operator,
            raw.source_ip_version,
        ) {
            (Some(value), Some(operator), Some(version)) => Some(AddrInfo {
                value,
                operator,
                version,
            }),
            _ => None,
        };

        let destination_port = match (raw.destination_port_value, raw.destination_port_operator) {
            (Some(value), Some(operator)) => Some(PortInfo { value, operator }),
            _ => None,
        };

        let destination_addr = match (
            raw.destination_ip_value,
            raw.destination_ip_operator,
            raw.destination_ip_version,
        ) {
            (Some(value), Some(operator), Some(version)) => Some(AddrInfo {
                value,
                operator,
                version,
            }),
            _ => None,
        };

        Ok(NatRule {
            disabled: raw.disabled,
            protocol: raw.protocol,
            source_inversed: raw.source_inversed,
            source_port,
            source_addr,
            source_type: raw.source_type,
            destination_inversed: raw.destination_inversed,
            destination_port,
            destination_addr,
            destination_type: raw.destination_type,
            description: raw.description,
            interface: raw.interface,
            redirect_ip: raw.redirect_ip,
            redirect_port: raw.redirect_port,
            order: raw.order,
            associated_rule_id: raw.associated_rule_id,
            table: raw.table.unwrap_or_default(),
            chain: raw.chain.unwrap_or_default(),
            family: raw.family.unwrap_or_default(),
        })
    }
}
