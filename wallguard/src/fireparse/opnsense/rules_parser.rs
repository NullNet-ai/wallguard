use crate::{opnsense::enpoint_parser::EndpointParser, Rule};
use roxmltree::{Document, Node};

pub struct OpnSenseRulesParser {}

impl OpnSenseRulesParser {
    pub fn parse(document: &Document) -> Vec<Rule> {
        let mut rules = vec![];

        if let Some(filter) = document
            .descendants()
            .find(|e| e.has_tag_name("opnsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("filter")))
        {
            rules.append(&mut OpnSenseRulesParser::parse_rules(filter, "filter"));
        }

        if let Some(nat) = document
            .descendants()
            .find(|e| e.has_tag_name("opnsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("nat")))
        {
            rules.append(&mut OpnSenseRulesParser::parse_rules(nat, "nat"));
        }

        rules
    }

    fn parse_rules(node: Node<'_, '_>, rule_type: &str) -> Vec<Rule> {
        let mut rules = Vec::new();

        for (index, rule) in (0_u64..).zip(node.children().filter(|e| e.has_tag_name("rule"))) {
            let disabled = rule.children().any(|e| e.has_tag_name("disabled"));

            let policy = rule
                .children()
                .find(|e| e.has_tag_name("type"))
                .and_then(|e| e.text())
                .unwrap_or("pass")
                .to_string();

            let ipprotocol = rule
                .children()
                .find(|e| e.has_tag_name("ipprotocol"))
                .and_then(|e| e.text())
                .unwrap_or("*");

            let protocol = rule
                .children()
                .find(|e| e.has_tag_name("protocol"))
                .and_then(|e| e.text())
                .unwrap_or("any");

            let description = rule
                .children()
                .find(|e| e.has_tag_name("descr"))
                .and_then(|e| e.text())
                .unwrap_or("")
                .to_string();

            let interface = rule
                .children()
                .find(|e| e.has_tag_name("interface"))
                .and_then(|e| e.text())
                .unwrap_or("none")
                .to_string();

            let (source_addr, source_port, source_type, source_inversed) =
                EndpointParser::parse(rule.children().find(|e| e.has_tag_name("source")));

            let (destination_addr, destination_port, destination_type, destination_inversed) =
                EndpointParser::parse(rule.children().find(|e| e.has_tag_name("destination")));

            rules.push(Rule {
                disabled,
                r#type: rule_type.to_string(),
                protocol: format!("{}/{}", ipprotocol, protocol),
                policy,
                description,
                source_port,
                source_addr,
                source_type,
                source_inversed,
                destination_addr,
                destination_port,
                destination_type,
                destination_inversed,
                interface,
                order: index,
            });
        }

        rules
    }
}
