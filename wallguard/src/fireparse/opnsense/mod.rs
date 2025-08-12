use nullnet_liberror::{location, Error, ErrorHandler, Location};
use wallguard_common::{
    os_if::InterfaceSnapshot,
    protobuf::wallguard_models::{Configuration, FilterRule},
};
use xmltree::Element;

use crate::fireparse::opnsense::{
    aliases_parser::OpnSenseAliasesParser, hostname_parser::OpnSenseHostnameParser,
    interfaces_parser::OpnSenseInterfacesParser, rules_parser::OpnSenseRulesParser,
    ssh_parser::OpnSenseSSHParser, webgui_parser::OpnSenseWebGuiParser,
};

mod aliases_parser;
mod endpoint_parser;
mod hostname_parser;
mod interfaces_parser;
mod rules_parser;
mod ssh_parser;
mod webgui_parser;

pub struct OpnSenseParser {}

impl OpnSenseParser {
    pub fn parse(data: &str) -> Result<Configuration, Error> {
        let document = Element::parse(data.as_bytes()).handle_err(location!())?;
        let interfaces = InterfaceSnapshot::take_all();
        let (filter_rules, nat_rules) = OpnSenseRulesParser::parse(&document);

        Ok(Configuration {
            digest: format!("{:x}", md5::compute(data)),
            aliases: OpnSenseAliasesParser::parse(&document),
            interfaces: OpnSenseInterfacesParser::parse(&document, interfaces),
            hostname: OpnSenseHostnameParser::parse(&document),
            gui_protocol: OpnSenseWebGuiParser::parse(&document, "https"),
            ssh_config: Some(OpnSenseSSHParser::parse(&document)),
            filter_rules,
            nat_rules,
        })
    }

    pub fn convert_filter_rule(rule: FilterRule) -> Element {
        OpnSenseRulesParser::filter_rule_to_element(rule)
    }
}
