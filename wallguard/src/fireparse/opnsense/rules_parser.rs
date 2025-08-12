use wallguard_common::protobuf::wallguard_models::{FilterRule, NatRule};
use xmltree::{Element, XMLNode};

use crate::fireparse::opnsense::endpoint_parser::EndpointParser;

pub struct OpnSenseRulesParser {}

impl OpnSenseRulesParser {
    pub fn filter_rule_to_element(rule: FilterRule) -> Element {
        let mut rule_elem = Element::new("rule");

        if rule.disabled {
            rule_elem
                .children
                .push(XMLNode::Element(Element::new("disabled")));
        }

        let mut type_elem = Element::new("type");
        type_elem.children.push(XMLNode::Text(rule.policy));
        rule_elem.children.push(XMLNode::Element(type_elem));

        let mut parts = rule.protocol.splitn(2, '/');
        let ipprotocol = parts.next().unwrap_or("inet46");
        let protocol = parts.next().unwrap_or("any");

        let mut ipproto_elem = Element::new("ipprotocol");
        ipproto_elem
            .children
            .push(XMLNode::Text(ipprotocol.to_string()));
        rule_elem.children.push(XMLNode::Element(ipproto_elem));

        if protocol != "any" {
            let mut proto_elem = Element::new("protocol");
            proto_elem
                .children
                .push(XMLNode::Text(protocol.to_string()));
            rule_elem.children.push(XMLNode::Element(proto_elem));
        }

        if !rule.description.is_empty() {
            let mut descr_elem = Element::new("descr");
            descr_elem.children.push(XMLNode::CData(rule.description));
            rule_elem.children.push(XMLNode::Element(descr_elem));
        }

        let mut iface_elem = Element::new("interface");
        iface_elem.children.push(XMLNode::Text(rule.interface));
        rule_elem.children.push(XMLNode::Element(iface_elem));

        let source_elem = EndpointParser::to_element(
            "source",
            &rule.source_addr,
            &rule.source_port,
            &rule.source_type,
            rule.source_inversed,
        );
        rule_elem.children.push(XMLNode::Element(source_elem));

        let destination_elem = EndpointParser::to_element(
            "destination",
            &rule.destination_addr,
            &rule.destination_port,
            &rule.destination_type,
            rule.destination_inversed,
        );
        rule_elem.children.push(XMLNode::Element(destination_elem));

        let mut tracker_elem = Element::new("tracker");
        tracker_elem
            .children
            .push(XMLNode::Text(rule.id.to_string()));
        rule_elem.children.push(XMLNode::Element(tracker_elem));

        rule_elem
    }

    pub fn parse(root: &Element) -> (Vec<FilterRule>, Vec<NatRule>) {
        let mut filter_rules = vec![];
        let mut nat_rules = vec![];

        if let Some(filter) = root.get_child("filter") {
            filter_rules.append(&mut OpnSenseRulesParser::parse_filter_rules(filter));
        }

        if let Some(nat) = root.get_child("nat") {
            nat_rules.append(&mut OpnSenseRulesParser::parse_nat_rules(nat));
        }

        (filter_rules, nat_rules)
    }

    fn parse_filter_rules(parent: &Element) -> Vec<FilterRule> {
        let mut rules = vec![];

        for (index, rule_node) in parent
            .children
            .iter()
            .filter_map(|node| match node {
                XMLNode::Element(e) if e.name == "rule" => Some(e),
                _ => None,
            })
            .enumerate()
        {
            let disabled = rule_node.get_child("disabled").is_some();

            let policy = rule_node
                .get_child("type")
                .and_then(|e| e.get_text())
                .unwrap_or("pass".into())
                .to_string();

            let ipprotocol = rule_node
                .get_child("ipprotocol")
                .and_then(|e| e.get_text())
                .unwrap_or("*".into())
                .to_string();

            let protocol = rule_node
                .get_child("protocol")
                .and_then(|e| e.get_text())
                .unwrap_or("any".into())
                .to_string();

            let description = rule_node
                .get_child("descr")
                .and_then(|e| e.get_text())
                .unwrap_or("".into())
                .to_string();

            let interface = rule_node
                .get_child("interface")
                .and_then(|e| e.get_text())
                .unwrap_or("none".into())
                .to_string();

            let (source_addr, source_port, source_type, source_inversed) =
                EndpointParser::parse(rule_node.get_child("source"));

            let (destination_addr, destination_port, destination_type, destination_inversed) =
                EndpointParser::parse(rule_node.get_child("destination"));

            rules.push(FilterRule {
                disabled,
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
                order: index as u32,
                // @TODO:
                id: index as u32,
            });
        }

        rules
    }

    fn parse_nat_rules(node: &Element) -> Vec<NatRule> {
        let mut rules = Vec::new();

        for (index, child) in node
            .children
            .iter()
            .filter_map(|c| match c {
                XMLNode::Element(e) if e.name == "rule" => Some(e),
                _ => None,
            })
            .enumerate()
        {
            let disabled = child.get_child("disabled").is_some();

            let ipprotocol = child
                .get_child("ipprotocol")
                .and_then(|e| e.get_text())
                .unwrap_or("*".into());

            let protocol = child
                .get_child("protocol")
                .and_then(|e| e.get_text())
                .unwrap_or("any".into());

            let description = child
                .get_child("descr")
                .and_then(|e| e.get_text())
                .unwrap_or("".into())
                .to_string();

            let interface = child
                .get_child("interface")
                .and_then(|e| e.get_text())
                .unwrap_or("none".into())
                .to_string();

            let (source_addr, source_port, source_type, source_inversed) =
                EndpointParser::parse(child.get_child("source"));

            let (destination_addr, destination_port, destination_type, destination_inversed) =
                EndpointParser::parse(child.get_child("destination"));

            let redirect_ip = child
                .get_child("target")
                .and_then(|e| e.get_text())
                .unwrap_or("none".into())
                .to_string();

            let redirect_port = child
                .get_child("local-port")
                .and_then(|e| e.get_text())
                .and_then(|text| text.parse::<u32>().ok())
                .unwrap_or(0);

            rules.push(NatRule {
                disabled,
                protocol: format!("{}/{}", ipprotocol, protocol),
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
                order: index as u32,
                redirect_ip,
                redirect_port,
            });
        }

        rules
    }
}
