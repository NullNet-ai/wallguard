use nftables::schema::Nftables;
use nullnet_liberror::Error;
use wallguard_common::protobuf::wallguard_models::Configuration;

use crate::fireparse::nft::{
    hostmane_parser::NftablesHostnameParser, rules_parser::NftablesRulesParser,
};

mod hostmane_parser;
mod rules_parser;
mod utils;

pub struct NftablesParser;

impl NftablesParser {
    pub fn parse(tables: Nftables<'_>) -> Result<Configuration, Error> {
        let (filter_rules, nat_rules) = NftablesRulesParser::parse(&tables);

        Ok(Configuration {
            digest: "todo".to_string(),
            aliases: vec![],
            filter_rules,
            nat_rules,
            interfaces: vec![],
            hostname: NftablesHostnameParser::parse()?,
            gui_protocol: String::new(),
            ssh_config: None,
        })
    }
}
