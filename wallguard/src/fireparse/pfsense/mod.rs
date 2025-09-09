use crate::utilities::system;
use aliases_parser::PfSenseAliasesParser;
use hostname_parser::PfSenseHostnameParser;
use interfaces_parser::PfSenseInterfacesParser;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use rules_parser::PfSenseRulesParser;
use ssh_parser::PfSenseSSHParser;
use wallguard_common::{
    os_if::InterfaceSnapshot,
    protobuf::wallguard_models::{Alias, Configuration, FilterRule, NatRule},
};
use webgui_parser::PfSenseWebGuiParser;
use xmltree::Element;

mod aliases_parser;
mod endpoint_parser;
mod hostname_parser;
mod interfaces_parser;
mod rules_parser;
mod ssh_parser;
mod webgui_parser;

pub struct PfSenseParser {}

impl PfSenseParser {
    pub fn parse(data: &str) -> Result<Configuration, Error> {
        let document = Element::parse(data.as_bytes()).handle_err(location!())?;
        let interfaces = InterfaceSnapshot::take_all();
        let (filter_rules, nat_rules) = PfSenseRulesParser::parse(&document);

        Ok(Configuration {
            digest: format!("{:x}", md5::compute(data)),
            aliases: PfSenseAliasesParser::parse(&document),
            interfaces: PfSenseInterfacesParser::parse(&document, interfaces),
            hostname: PfSenseHostnameParser::parse(&document),
            gui_protocol: PfSenseWebGuiParser::parse(&document, "https"),
            ssh_config: Some(PfSenseSSHParser::parse(&document)),
            filter_rules,
            nat_rules,
            tables: vec![],
            chains: vec![],
        })
    }

    pub async fn create_filter_rule(rule: FilterRule) -> Result<(), Error> {
        let element = PfSenseRulesParser::filter_rule_to_element(rule);

        let content = tokio::fs::read("/conf/config.xml")
            .await
            .handle_err(location!())?;

        let mut document = Element::parse(content.as_slice()).handle_err(location!())?;

        let rules_node = document
            .get_mut_child("filter")
            .ok_or("Malformed config.xml file")
            .handle_err(location!())?;

        rules_node.children.push(xmltree::XMLNode::Element(element));

        let mut buffer = Vec::new();
        document.write(&mut buffer).handle_err(location!())?;
        tokio::fs::write("/conf/config.xml", buffer)
            .await
            .handle_err(location!())?;

        system::reload_configuraion().await
    }

    pub async fn create_nat_rule(rule: NatRule) -> Result<(), Error> {
        let element = PfSenseRulesParser::nat_rule_to_element(rule);

        let content = tokio::fs::read("/conf/config.xml")
            .await
            .handle_err(location!())?;

        let mut document = Element::parse(content.as_slice()).handle_err(location!())?;

        let rules_node = document
            .get_mut_child("nat")
            .ok_or("Malformed config.xml file")
            .handle_err(location!())?;

        rules_node.children.push(xmltree::XMLNode::Element(element));

        let mut buffer = Vec::new();
        document.write(&mut buffer).handle_err(location!())?;
        tokio::fs::write("/conf/config.xml", buffer)
            .await
            .handle_err(location!())?;

        system::reload_configuraion().await
    }

    pub async fn create_alias(alias: Alias) -> Result<(), Error> {
        let element = PfSenseAliasesParser::to_element(alias);

        let content = tokio::fs::read("/conf/config.xml")
            .await
            .handle_err(location!())?;

        let mut document = Element::parse(content.as_slice()).handle_err(location!())?;

        let aliases_node = document
            .get_mut_child("aliases")
            .ok_or("Malformed config.xml")
            .handle_err(location!())?;

        aliases_node
            .children
            .push(xmltree::XMLNode::Element(element));

        let mut buffer = Vec::new();
        document.write(&mut buffer).handle_err(location!())?;
        tokio::fs::write("/conf/config.xml", buffer)
            .await
            .handle_err(location!())?;

        system::reload_configuraion().await
    }
}
