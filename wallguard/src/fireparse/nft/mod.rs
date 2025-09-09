use nftables::schema::{NfListObject, NfObject, Nftables, Rule};
use nullnet_liberror::Error;
use wallguard_common::protobuf::wallguard_models::{Configuration, FilterRule};

use crate::fireparse::nft::{
    addr_helper::AddrHelper, hostmane_parser::NftablesHostnameParser, port_helper::PortHelper,
    rules_parser::NftablesRulesParser, utils::NftDirection,
};

mod hostmane_parser;
mod rules_parser;
mod utils;

mod addr_helper;
mod interface_helper;
mod ip_protocol_helper;
mod l4_protocol_helper;
mod nat_helper;
mod policy_helper;
mod port_helper;

pub struct NftablesParser;

impl NftablesParser {
    pub fn parse(tables: Nftables<'_>, digest: String) -> Result<Configuration, Error> {
        let (filter_rules, nat_rules) = NftablesRulesParser::parse(&tables);
        let (tables, chains) = NftablesParser::collect_tables_and_chains(&tables);

        Ok(Configuration {
            digest,
            aliases: vec![],
            filter_rules,
            nat_rules,
            interfaces: vec![],
            hostname: NftablesHostnameParser::parse()?,
            gui_protocol: String::new(),
            ssh_config: None,
            tables,
            chains,
        })
    }

    pub fn collect_tables_and_chains(tables: &Nftables<'_>) -> (Vec<String>, Vec<String>) {
        let mut tables_list = vec![];
        let mut chains_list = vec![];

        for object in tables.objects.iter() {
            if let NfObject::ListObject(NfListObject::Table(table)) = object {
                tables_list.push(table.name.to_string());
            }

            if let NfObject::ListObject(NfListObject::Chain(chain)) = object {
                chains_list.push(chain.name.to_string());
            }
        }

        (tables_list, chains_list)
    }
}
