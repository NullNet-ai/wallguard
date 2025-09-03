use crate::fireparse::nft::utils::{extract_addr_info, extract_port_info, NftDirection};
use nftables::schema::Nftables;
use wallguard_common::protobuf::wallguard_models::{FilterRule, NatRule};

pub struct NftablesRulesParser;

impl NftablesRulesParser {
    pub fn parse(tables: &Nftables<'_>) -> (Vec<FilterRule>, Vec<NatRule>) {
        for object in tables.objects.iter() {
            match object {
                nftables::schema::NfObject::ListObject(nf_list_object) => match nf_list_object {
                    nftables::schema::NfListObject::Rule(rule) => {
                        println!("{:?}", rule);
                        let source_addr = extract_addr_info(rule, NftDirection::Source);
                        let source_port = extract_port_info(rule, NftDirection::Source);

                        let destination_addr = extract_addr_info(rule, NftDirection::Destination);
                        let destination_port = extract_port_info(rule, NftDirection::Destination);

                        if let Some(addr) = source_addr {
                            println!("Src: {:?}", addr);
                        }

                        if let Some(port) = source_port {
                            println!("Src p: {:?}", port);
                        }

                        if let Some(addr) = destination_addr {
                            println!("Dest: {:?}", addr);
                        }

                        if let Some(port) = destination_port {
                            println!("Dest p: {:?}", port);
                        }

                        println!()
                    }
                    _ => continue,
                },
                nftables::schema::NfObject::CmdObject(_) => continue,
            }
        }

        (vec![], vec![])
    }
}
