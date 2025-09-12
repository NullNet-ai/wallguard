use crate::fireparse::nft::{
    aliases_parser::NftablesAliasesParser, hostmane_parser::NftablesHostnameParser,
    rules_parser::NftablesRulesParser,
};
use nftables::{
    batch::Batch,
    schema::{NfCmd, NfListObject, NfObject, Nftables},
};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use wallguard_common::protobuf::wallguard_models::{Alias, Configuration, FilterRule, NatRule};

mod addr_helper;
mod aliases_parser;
mod hostmane_parser;
mod interface_helper;
mod ip_protocol_helper;
mod l4_protocol_helper;
mod nat_helper;
mod policy_helper;
mod port_helper;
mod rules_parser;
mod utils;

pub struct NftablesParser;

impl NftablesParser {
    pub fn parse(tables: Nftables<'_>, digest: String) -> Result<Configuration, Error> {
        let (filter_rules, nat_rules) = NftablesRulesParser::parse(&tables);
        let aliases = NftablesAliasesParser::parse(&tables);
        let (tables, chains) = NftablesParser::collect_tables_and_chains(&tables);

        Ok(Configuration {
            digest,
            aliases,
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

    pub async fn create_filter_rule(rule: FilterRule) -> Result<(), Error> {
        let rule = NftablesRulesParser::convert_filter_rule(rule)?;

        let mut batch = Batch::new();
        batch.add_cmd(NfCmd::Add(NfListObject::Rule(rule)));

        tokio::task::spawn_blocking(move || {
            let table = batch.to_nftables();
            nftables::helper::apply_ruleset(&table)
        })
        .await
        .handle_err(location!())?
        .handle_err(location!())
    }

    pub async fn create_nat_rule(rule: NatRule) -> Result<(), Error> {
        let rule = NftablesRulesParser::convert_nat_rule(rule)?;

        let mut batch = Batch::new();
        batch.add_cmd(NfCmd::Add(NfListObject::Rule(rule)));

        tokio::task::spawn_blocking(move || {
            let table = batch.to_nftables();
            nftables::helper::apply_ruleset(&table)
        })
        .await
        .handle_err(location!())?
        .handle_err(location!())
    }

    pub async fn create_alias(alias: Alias) -> Result<(), Error> {
        let alias = NftablesAliasesParser::convert_alias(alias);
        let mut batch = Batch::new();
        batch.add_cmd(NfCmd::Add(NfListObject::Set(alias)));

        tokio::task::spawn_blocking(move || {
            let table = batch.to_nftables();
            nftables::helper::apply_ruleset(&table)
        })
        .await
        .handle_err(location!())?
        .handle_err(location!())
    }
}
