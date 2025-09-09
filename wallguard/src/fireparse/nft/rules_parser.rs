use crate::fireparse::nft::{
    addr_helper::AddrHelper, interface_helper::InterfaceHelper,
    ip_protocol_helper::IpProtocolHelper, l4_protocol_helper::L4ProtocolHelper,
    nat_helper::NatHelper, policy_helper::PolicyHelper, port_helper::PortHelper,
    utils::NftDirection,
};
use nftables::schema::{Nftables, Rule};
use nullnet_liberror::Error;
use wallguard_common::protobuf::wallguard_models::{FilterRule, NatRule};

pub struct NftablesRulesParser;

impl NftablesRulesParser {
    pub fn parse(tables: &Nftables<'_>) -> (Vec<FilterRule>, Vec<NatRule>) {
        let mut filter_rules = vec![];
        let mut nat_rules = vec![];

        for (index, object) in tables.objects.iter().enumerate() {
            match object {
                nftables::schema::NfObject::ListObject(nf_list_object) => match nf_list_object {
                    nftables::schema::NfListObject::Rule(rule) => {
                        let source_addr = AddrHelper::extract(rule, NftDirection::Source);
                        let source_port = PortHelper::extract(rule, NftDirection::Source);

                        let destination_addr = AddrHelper::extract(rule, NftDirection::Destination);
                        let destination_port = PortHelper::extract(rule, NftDirection::Destination);

                        let policy = PolicyHelper::extract(rule).unwrap_or(String::from("accept"));

                        let ip_protocol = IpProtocolHelper::extract(rule);
                        let l4_proto = L4ProtocolHelper::extract(rule);
                        let interface = InterfaceHelper::extract(rule);

                        if let Some((addr, port)) = NatHelper::extract(rule) {
                            nat_rules.push(NatRule {
                                disabled: false,
                                protocol: format!(
                                    "{}/{}",
                                    ip_protocol.unwrap_or("*".into()),
                                    l4_proto.unwrap_or("*".into())
                                ),
                                source_inversed: false,
                                source_port,
                                source_addr,
                                source_type: String::default(),
                                destination_inversed: false,
                                destination_port,
                                destination_addr,
                                destination_type: String::default(),
                                description: rule
                                    .comment
                                    .as_ref()
                                    .map(|v| v.to_string())
                                    .unwrap_or_default(),
                                interface: interface.unwrap_or("*".into()),
                                order: index as u32,
                                associated_rule_id: String::default(),
                                redirect_ip: addr.unwrap_or_default(),
                                redirect_port: port.unwrap_or_default(),
                                table: rule.table.to_string(),
                                chain: rule.chain.to_string(),
                            });
                        } else {
                            filter_rules.push(FilterRule {
                                disabled: false,
                                policy,
                                protocol: format!(
                                    "{}/{}",
                                    ip_protocol.unwrap_or("*".into()),
                                    l4_proto.unwrap_or("*".into())
                                ),
                                source_inversed: false,
                                source_port,
                                source_addr,
                                source_type: String::default(),
                                destination_inversed: false,
                                destination_port,
                                destination_addr,
                                destination_type: String::default(),
                                description: rule
                                    .comment
                                    .as_ref()
                                    .map(|v| v.to_string())
                                    .unwrap_or_default(),
                                interface: interface.unwrap_or("*".into()),
                                id: index as u32,
                                order: index as u32,
                                associated_rule_id: String::default(),
                                table: rule.table.to_string(),
                                chain: rule.chain.to_string(),
                            });
                        }
                    }
                    _ => continue,
                },
                nftables::schema::NfObject::CmdObject(_) => continue,
            }
        }

        (filter_rules, nat_rules)
    }

    pub fn convert_filter_rule(filter_rule: FilterRule) -> Result<Rule<'static>, Error> {
        let mut statements = vec![];

        let (ip_protocol, l4_protocol) = filter_rule.protocol.split_once("/").unwrap_or(("*", "*"));

        if let Some(stmt) = IpProtocolHelper::build(ip_protocol) {
            statements.push(stmt);
        }

        if !filter_rule.interface.is_empty() {
            let statement = InterfaceHelper::build(&filter_rule.interface);
            statements.push(statement);
        }

        if let Some(stmt) = filter_rule
            .source_addr
            .and_then(|source_addr| AddrHelper::build(&source_addr, NftDirection::Source))
        {
            statements.push(stmt);
        }

        if let Some(stmt) = filter_rule.source_port.and_then(|source_port| {
            PortHelper::build(&source_port, NftDirection::Source, l4_protocol)
        }) {
            statements.push(stmt);
        }

        if let Some(stmt) = filter_rule.destination_addr.and_then(|destination_addr| {
            AddrHelper::build(&destination_addr, NftDirection::Destination)
        }) {
            statements.push(stmt);
        }

        if let Some(stmt) = filter_rule.destination_port.and_then(|destination_port| {
            PortHelper::build(&destination_port, NftDirection::Destination, l4_protocol)
        }) {
            statements.push(stmt);
        }

        if let Some(stmt) = PolicyHelper::build(&filter_rule.policy) {
            statements.push(stmt);
        }

        Ok(Rule {
            table: filter_rule.table.into(),
            chain: filter_rule.chain.into(),
            expr: statements.into(),
            comment: Some(filter_rule.description.into()),
            ..Default::default()
        })
    }

    pub fn convert_nat_rule(nat_rule: NatRule) -> Result<Rule<'static>, Error> {
        let mut statements = vec![];

        let (ip_protocol, l4_protocol) = nat_rule.protocol.split_once("/").unwrap_or(("*", "*"));

        if let Some(stmt) = IpProtocolHelper::build(ip_protocol) {
            statements.push(stmt);
        }

        if !nat_rule.interface.is_empty() {
            let statement = InterfaceHelper::build(&nat_rule.interface);
            statements.push(statement);
        }

        if let Some(stmt) = nat_rule
            .source_addr
            .and_then(|source_addr| AddrHelper::build(&source_addr, NftDirection::Source))
        {
            statements.push(stmt);
        }

        if let Some(stmt) = nat_rule.source_port.and_then(|source_port| {
            PortHelper::build(&source_port, NftDirection::Source, l4_protocol)
        }) {
            statements.push(stmt);
        }

        if let Some(stmt) = nat_rule.destination_addr.and_then(|destination_addr| {
            AddrHelper::build(&destination_addr, NftDirection::Destination)
        }) {
            statements.push(stmt);
        }

        if let Some(stmt) = nat_rule.destination_port.and_then(|destination_port| {
            PortHelper::build(&destination_port, NftDirection::Destination, l4_protocol)
        }) {
            statements.push(stmt);
        }

        let nat_ip = Some(nat_rule.redirect_ip).filter(|value| !value.is_empty());
        let nat_port = Some(nat_rule.redirect_port).filter(|value| *value != 0);

        if let Some(stmt) = NatHelper::build(nat_ip, nat_port) {
            statements.push(stmt);
        }

        Ok(Rule {
            table: nat_rule.table.into(),
            chain: nat_rule.chain.into(),
            expr: statements.into(),
            comment: Some(nat_rule.description.into()),
            ..Default::default()
        })
    }
}
