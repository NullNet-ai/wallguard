use crate::fireparse::nft::utils::{
    extract_addr_info, extract_interface, extract_ip_protocol, extract_l4_protocol, extract_nat,
    extract_policy, extract_port_info, NftDirection,
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
                        println!("{:?}", rule);

                        let source_addr = extract_addr_info(rule, NftDirection::Source);
                        let source_port = extract_port_info(rule, NftDirection::Source);

                        let destination_addr = extract_addr_info(rule, NftDirection::Destination);
                        let destination_port = extract_port_info(rule, NftDirection::Destination);

                        let policy = extract_policy(rule).unwrap_or(String::from("accept"));

                        let ip_protocol = extract_ip_protocol(rule);
                        let l4_proto = extract_l4_protocol(rule);
                        let interface = extract_interface(rule);

                        if let Some((addr, port)) = extract_nat(rule) {
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
                                    .and_then(|v| Some(v.to_string()))
                                    .unwrap_or_default(),
                                interface: interface.unwrap_or("*".into()),
                                order: index as u32,
                                associated_rule_id: String::default(),
                                redirect_ip: addr.unwrap_or_default(),
                                redirect_port: port.unwrap_or_default(),
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
                                    .and_then(|v| Some(v.to_string()))
                                    .unwrap_or_default(),
                                interface: interface.unwrap_or("*".into()),
                                id: index as u32,
                                order: index as u32,
                                associated_rule_id: String::default(),
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
