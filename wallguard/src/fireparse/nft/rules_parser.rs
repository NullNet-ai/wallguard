use crate::fireparse::nft::{
    addr_helper::AddrHelper, interface_helper::InterfaceHelper,
    ip_protocol_helper::IpProtocolHelper, l4_protocol_helper::L4ProtocolHelper,
    nat_helper::NatHelper, policy_helper::PolicyHelper, port_helper::PortHelper,
    utils::NftDirection,
};
use nftables::schema::Nftables;
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
}
