use aliases_parser::PfSenseAliasesParser;
use hostname_parser::PfSenseHostnameParser;
use interfaces_parser::PfSenseInterfacesParser;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use rules_parser::PfSenseRulesParser;
use ssh_parser::PfSenseSSHParser;
use wallguard_common::{
    os_if::InterfaceSnapshot,
    protobuf::wallguard_models::{Configuration, FilterRule},
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
        })
    }

    pub fn convert_filter_rule(rule: FilterRule) -> Element {
        PfSenseRulesParser::filter_rule_to_element(rule)
    }
}
