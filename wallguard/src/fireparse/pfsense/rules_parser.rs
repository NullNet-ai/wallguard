use super::endpoint_parser::EndpointParser;
use wallguard_common::protobuf::wallguard_models::{AddrInfo, FilterRule, NatRule, PortInfo};
use xmltree::{Element, XMLNode};

pub struct PfSenseRulesParser;

impl PfSenseRulesParser {
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
            &rule.source_addr.map(|v| v.value).unwrap_or("*".into()),
            &rule.source_port.map(|v| v.value).unwrap_or("*".into()),
            &rule.source_type,
            rule.source_inversed,
        );
        rule_elem.children.push(XMLNode::Element(source_elem));

        let destination_elem = EndpointParser::to_element(
            "destination",
            &rule.destination_addr.map(|v| v.value).unwrap_or("*".into()),
            &rule.destination_port.map(|v| v.value).unwrap_or("*".into()),
            &rule.destination_type,
            rule.destination_inversed,
        );
        rule_elem.children.push(XMLNode::Element(destination_elem));

        let mut tracker_elem = Element::new("tracker");
        tracker_elem
            .children
            .push(XMLNode::Text(rule.id.to_string()));
        rule_elem.children.push(XMLNode::Element(tracker_elem));

        let mut associated_rule_id_elem = Element::new("associated-rule-id");
        associated_rule_id_elem
            .children
            .push(XMLNode::Text(rule.associated_rule_id));
        rule_elem
            .children
            .push(XMLNode::Element(associated_rule_id_elem));

        rule_elem
    }

    pub fn nat_rule_to_element(rule: NatRule) -> Element {
        let mut rule_elem = Element::new("rule");

        if rule.disabled {
            rule_elem
                .children
                .push(XMLNode::Element(Element::new("disabled")));
        }

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
            &rule.source_addr.map(|v| v.value).unwrap_or("*".into()),
            &rule.source_port.map(|v| v.value).unwrap_or("*".into()),
            &rule.source_type,
            rule.source_inversed,
        );
        rule_elem.children.push(XMLNode::Element(source_elem));

        let destination_elem = EndpointParser::to_element(
            "destination",
            &rule.destination_addr.map(|v| v.value).unwrap_or("*".into()),
            &rule.destination_port.map(|v| v.value).unwrap_or("*".into()),
            &rule.destination_type,
            rule.destination_inversed,
        );
        rule_elem.children.push(XMLNode::Element(destination_elem));

        let mut associated_rule_id_elem = Element::new("associated-rule-id");
        associated_rule_id_elem
            .children
            .push(XMLNode::Text(rule.associated_rule_id));
        rule_elem
            .children
            .push(XMLNode::Element(associated_rule_id_elem));

        let mut target_elem = Element::new("target");
        target_elem.children.push(XMLNode::Text(rule.redirect_ip));
        rule_elem.children.push(XMLNode::Element(target_elem));

        let mut port_elem = Element::new("local-port");
        port_elem
            .children
            .push(XMLNode::Text(rule.redirect_port.to_string()));
        rule_elem.children.push(XMLNode::Element(port_elem));

        rule_elem
    }

    pub fn parse(document: &Element) -> (Vec<FilterRule>, Vec<NatRule>) {
        let mut filter_rules = Vec::new();
        let mut nat_rules = Vec::new();

        if let Some(filter) = document.get_child("filter") {
            filter_rules.extend(Self::parse_filter_rules(filter));
        }
        if let Some(nat) = document.get_child("nat") {
            nat_rules.extend(Self::parse_nat_rules(nat));
        }

        (filter_rules, nat_rules)
    }

    fn parse_filter_rules(node: &Element) -> Vec<FilterRule> {
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

            let policy = child
                .get_child("type")
                .and_then(|e| e.get_text())
                .unwrap_or("pass".into())
                .to_string();

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

            let id = child
                .get_child("tracker")
                .and_then(|e| e.get_text())
                .and_then(|text| text.parse::<u32>().ok())
                .unwrap_or(0);

            let associated_rule_id = child
                .get_child("associated-rule-id")
                .and_then(|e| e.get_text())
                .unwrap_or("".into())
                .to_string();

            rules.push(FilterRule {
                disabled,
                protocol: format!("{ipprotocol}/{protocol}"),
                policy,
                description,
                source_port: Some(PortInfo {
                    value: source_port,
                    ..Default::default()
                }),
                source_addr: Some(AddrInfo {
                    value: source_addr,
                    ..Default::default()
                }),
                source_type,
                source_inversed,
                destination_addr: Some(AddrInfo {
                    value: destination_addr,
                    ..Default::default()
                }),
                destination_port: Some(PortInfo {
                    value: destination_port,
                    ..Default::default()
                }),
                destination_type,
                destination_inversed,
                interface,
                order: index as u32,
                id,
                associated_rule_id,
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

            let associated_rule_id = child
                .get_child("associated-rule-id")
                .and_then(|e| e.get_text())
                .unwrap_or("".into())
                .to_string();

            rules.push(NatRule {
                disabled,
                protocol: format!("{ipprotocol}/{protocol}",),
                description,
                source_port: Some(PortInfo {
                    value: source_port,
                    operator: String::default(),
                }),
                source_addr: Some(AddrInfo {
                    version: 0,
                    value: source_addr,
                    operator: String::default(),
                }),
                source_type,
                source_inversed,
                destination_addr: Some(AddrInfo {
                    version: 0,
                    value: destination_addr,
                    operator: String::default(),
                }),
                destination_port: Some(PortInfo {
                    value: destination_port,
                    operator: String::default(),
                }),
                destination_type,
                destination_inversed,
                interface,
                order: index as u32,
                redirect_ip,
                redirect_port,
                associated_rule_id,
            });
        }

        rules
    }
}
#[cfg(test)]
mod tests {
    use super::PfSenseRulesParser;
    use wallguard_common::protobuf::wallguard_models::{AddrInfo, FilterRule, PortInfo};
    use xmltree::{Element, XMLNode};

    fn find_child_text(element: &Element, name: &str) -> Option<String> {
        element
            .get_child(name)
            .and_then(|e| e.get_text())
            .map(|s| s.to_string())
    }

    fn has_child(element: &Element, name: &str) -> bool {
        element.get_child(name).is_some()
    }

    #[test]
    fn test_parse_filter_rules() {
        let xml = r#"
        <pfsense>
            <filter>
                <rule>
                    <type>pass</type>
                    <ipprotocol>inet</ipprotocol>
                    <descr>Default allow LAN to any rule</descr>
                    <interface>lan</interface>
                    <source>
                        <network>lan</network>
                    </source>
                    <destination>
                        <any/>
                    </destination>
                </rule>
            </filter>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let (rules, _) = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].disabled, false);
        assert_eq!(rules[0].policy, "pass");
        assert_eq!(rules[0].protocol, "inet/any");
        assert_eq!(rules[0].description, "Default allow LAN to any rule");
        assert_eq!(rules[0].source_addr.as_ref().unwrap().value, "lan");
        assert_eq!(rules[0].source_port.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].source_type, "network");
        assert_eq!(rules[0].source_inversed, false);
        assert_eq!(rules[0].destination_addr.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].destination_port.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].destination_type, "address");
        assert_eq!(rules[0].destination_inversed, false);
        assert_eq!(rules[0].interface, "lan");
        assert_eq!(rules[0].order, 0);
    }

    #[test]
    fn test_parse_nat_rules() {
        let xml = r#"
        <pfsense>
            <nat>
                <rule>
                    <source>
                        <any></any>
                    </source>
                    <destination>
                        <network>wanip</network>
                        <port>8091</port>
                    </destination>
                    <ipprotocol>inet6</ipprotocol>
                    <protocol>tcp</protocol>
                    <target>172.16.70.20</target>
                    <local-port>8080</local-port>
                    <interface>wan</interface>
                    <descr>NAT Rule</descr>
                </rule>
            </nat>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let (_, rules) = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].disabled, false);
        assert_eq!(rules[0].protocol, "inet6/tcp");
        assert_eq!(rules[0].description, "NAT Rule");
        assert_eq!(rules[0].source_addr.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].source_port.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].source_type, "address");
        assert_eq!(rules[0].source_inversed, false);
        assert_eq!(rules[0].destination_addr.as_ref().unwrap().value, "wanip");
        assert_eq!(rules[0].destination_port.as_ref().unwrap().value, "8091");
        assert_eq!(rules[0].destination_type, "network");
        assert_eq!(rules[0].destination_inversed, false);
        assert_eq!(rules[0].interface, "wan");
        assert_eq!(rules[0].order, 0);
    }

    #[test]
    fn test_parse_multiple_rules() {
        let xml = r#"
        <pfsense>
            <filter>
                <rule>
                    <disabled></disabled>
                    <type>pass</type>
                    <ipprotocol>inet</ipprotocol>
                    <descr>Allow LAN</descr>
                    <interface>lan</interface>
                    <source>
                        <network>lan</network>
                    </source>
                    <destination>
                        <any/>
                    </destination>
                </rule>
            </filter>
            <nat>
                <rule>
                    <disabled/>
                    <source>
                        <any></any>
                    </source>
                    <destination>
                        <network>wanip</network>
                        <port>8091</port>
                    </destination>
                    <ipprotocol>inet</ipprotocol>
                    <protocol>tcp</protocol>
                    <target>172.16.70.20</target>
                    <local-port>8080</local-port>
                    <interface>wan</interface>
                    <descr>NAT Rule</descr>
                </rule>
            </nat>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let (frules, nrules) = PfSenseRulesParser::parse(&doc);

        assert_eq!(frules.len(), 1);

        // Verify the first rule (Filter)
        assert_eq!(frules[0].disabled, true);
        assert_eq!(frules[0].policy, "pass");
        assert_eq!(frules[0].protocol, "inet/any");
        assert_eq!(frules[0].description, "Allow LAN");
        assert_eq!(frules[0].source_addr.as_ref().unwrap().value, "lan");
        assert_eq!(frules[0].source_port.as_ref().unwrap().value, "*");
        assert_eq!(frules[0].source_type, "network");
        assert_eq!(frules[0].source_inversed, false);
        assert_eq!(frules[0].destination_addr.as_ref().unwrap().value, "*");
        assert_eq!(frules[0].destination_port.as_ref().unwrap().value, "*");
        assert_eq!(frules[0].destination_type, "address");
        assert_eq!(frules[0].destination_inversed, false);
        assert_eq!(frules[0].interface, "lan");
        assert_eq!(frules[0].order, 0);

        assert_eq!(nrules.len(), 1);

        // Verify the second rule (NAT)
        assert_eq!(nrules[0].disabled, true);
        assert_eq!(nrules[0].protocol, "inet/tcp");
        assert_eq!(nrules[0].description, "NAT Rule");
        assert_eq!(nrules[0].source_addr.as_ref().unwrap().value, "*");
        assert_eq!(nrules[0].source_port.as_ref().unwrap().value, "*");
        assert_eq!(nrules[0].destination_addr.as_ref().unwrap().value, "wanip");
        assert_eq!(nrules[0].destination_port.as_ref().unwrap().value, "8091");
        assert_eq!(nrules[0].destination_type, "network");
        assert_eq!(nrules[0].destination_inversed, false);
        assert_eq!(nrules[0].interface, "wan");
        assert_eq!(nrules[0].order, 0);
        assert_eq!(nrules[0].redirect_ip, "172.16.70.20");
        assert_eq!(nrules[0].redirect_port, 8080);
    }

    #[test]
    fn test_parse_missing_optional_fields() {
        let xml = r#"
        <pfsense>
            <filter>
                <rule>
                    <type>reject</type>
                    <ipprotocol>inet</ipprotocol>
                    <descr>Block traffic</descr>
                    <source>
                        <any></any>
                    </source>
                    <destination>
                        <address>restricted_zone</address>
                    </destination>
                    <interface>opt1</interface>
                </rule>
            </filter>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let (rules, _) = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].disabled, false);
        assert_eq!(rules[0].policy, "reject");
        assert_eq!(rules[0].protocol, "inet/any");
        assert_eq!(rules[0].description, "Block traffic");
        assert_eq!(rules[0].source_addr.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].source_port.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].source_type, "address");
        assert_eq!(rules[0].source_inversed, false);
        assert_eq!(
            rules[0].destination_addr.as_ref().unwrap().value,
            "restricted_zone"
        );
        assert_eq!(rules[0].destination_port.as_ref().unwrap().value, "*");
        assert_eq!(rules[0].destination_type, "address");
        assert_eq!(rules[0].destination_inversed, false);
        assert_eq!(rules[0].interface, "opt1");
        assert_eq!(rules[0].order, 0);
    }

    #[test]
    fn test_parse_ordering() {
        let xml = r#"
        <pfsense>
            <filter>
                <rule>
                    <type>reject</type>
                    <interface>opt1</interface>
                </rule>
                <rule>
                    <type>reject</type>
                    <interface>opt1</interface>
                </rule>
                <rule>
                    <type>reject</type>
                    <interface>opt1</interface>
                </rule>
            </filter>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let (rules, _) = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].order, 0);
        assert_eq!(rules[1].order, 1);
        assert_eq!(rules[2].order, 2);
    }

    #[test]
    fn test_filter_rule_to_element_full() {
        let rule = FilterRule {
            disabled: true,
            policy: "block".to_string(),
            protocol: "inet/tcp".to_string(),
            description: "Block SSH".to_string(),
            source_addr: Some(AddrInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            source_port: Some(PortInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            source_type: "address".to_string(),
            source_inversed: false,
            destination_addr: Some(AddrInfo {
                value: "192.168.1.100".to_string(),
                ..Default::default()
            }),
            destination_port: Some(PortInfo {
                value: "22".to_string(),
                ..Default::default()
            }),
            destination_type: "address".to_string(),
            destination_inversed: false,
            interface: "lan".to_string(),
            order: 0,
            id: 42,
            associated_rule_id: "qwerty".to_string(),
        };

        let elem = PfSenseRulesParser::filter_rule_to_element(rule);

        assert_eq!(elem.name, "rule");
        assert!(has_child(&elem, "disabled"));
        assert_eq!(find_child_text(&elem, "type").unwrap(), "block");
        assert_eq!(find_child_text(&elem, "ipprotocol").unwrap(), "inet");
        assert_eq!(find_child_text(&elem, "protocol").unwrap(), "tcp");
        assert_eq!(find_child_text(&elem, "interface").unwrap(), "lan");
        assert_eq!(find_child_text(&elem, "tracker").unwrap(), "42");
        assert_eq!(
            find_child_text(&elem, "associated-rule-id").unwrap(),
            "qwerty"
        );

        let destination = elem.get_child("destination").unwrap();
        assert_eq!(
            find_child_text(destination, "address").unwrap(),
            "192.168.1.100"
        );
        assert_eq!(find_child_text(destination, "port").unwrap(), "22");
    }

    #[test]
    fn test_filter_rule_to_element_omits_any_protocol() {
        let rule = FilterRule {
            disabled: false,
            policy: "pass".to_string(),
            protocol: "inet/any".to_string(),
            description: "".to_string(),
            source_addr: Some(AddrInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            source_port: Some(PortInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            source_type: "address".to_string(),
            source_inversed: false,
            destination_addr: Some(AddrInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            destination_port: Some(PortInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            destination_type: "address".to_string(),
            destination_inversed: false,
            interface: "wan".to_string(),
            order: 1,
            id: 100,
            associated_rule_id: "qwerty".to_string(),
        };

        let elem = PfSenseRulesParser::filter_rule_to_element(rule);

        assert_eq!(find_child_text(&elem, "ipprotocol").unwrap(), "inet");
        assert!(
            elem.get_child("protocol").is_none(),
            "protocol should be omitted"
        );
    }

    #[test]
    fn test_filter_rule_to_element_includes_description_cdata() {
        let rule = FilterRule {
            disabled: false,
            policy: "pass".to_string(),
            protocol: "inet/tcp".to_string(),
            description: "Allow HTTP traffic".to_string(),
            source_addr: Some(AddrInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            source_port: Some(PortInfo {
                value: "*".to_string(),
                ..Default::default()
            }),
            destination_addr: Some(AddrInfo {
                value: "10.0.0.1".to_string(),
                ..Default::default()
            }),
            destination_port: Some(PortInfo {
                value: "80".to_string(),
                ..Default::default()
            }),
            source_type: "address".to_string(),
            source_inversed: false,
            destination_type: "address".to_string(),
            destination_inversed: false,
            interface: "wan".to_string(),
            order: 2,
            id: 55,
            associated_rule_id: "qwerty".to_string(),
        };

        let elem = PfSenseRulesParser::filter_rule_to_element(rule);
        let descr_elem = elem.get_child("descr").unwrap();

        if let Some(XMLNode::CData(cdata)) = descr_elem.children.first() {
            assert_eq!(cdata, "Allow HTTP traffic");
        } else {
            panic!("Description should be wrapped in CDATA");
        }
    }
}
