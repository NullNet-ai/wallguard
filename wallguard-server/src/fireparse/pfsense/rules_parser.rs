use roxmltree::{Document, Node};

use super::endpoint_parser::EndpointParser;
use crate::fireparse::Rule;

/// A parser for extracting firewall and NAT rules from a pfSense XML configuration.
pub struct PfSenseRulesParser {}

impl PfSenseRulesParser {
    /// Parses a pfSense XML document to extract `filter` and `nat` rules.
    ///
    /// # Arguments
    /// * `document` - A reference to a `Document` containing the pfSense configuration.
    ///
    /// # Returns
    /// A `Vec<Rule>` containing all parsed rules from the `<filter>` and `<nat>` sections.
    ///
    /// The function first looks for a `<pfsense>` node and then extracts `<filter>` and `<nat>` rules.
    pub fn parse(document: &Document) -> Vec<Rule> {
        let mut rules = vec![];

        if let Some(filter) = document
            .descendants()
            .find(|e| e.has_tag_name("pfsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("filter")))
        {
            rules.append(&mut PfSenseRulesParser::parse_rules(filter, "filter"));
        }

        if let Some(nat) = document
            .descendants()
            .find(|e| e.has_tag_name("pfsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("nat")))
        {
            rules.append(&mut PfSenseRulesParser::parse_rules(nat, "nat"));
        }

        rules
    }

    /// Generic function to parse rules from a given XML node.
    ///
    /// # Arguments
    /// * `node` - A `Node` representing either the `<filter>` or `<nat>` section.
    /// * `rule_type` - A `&str` indicating the type of rule (`"filter"` or `"nat"`).
    ///
    /// # Returns
    /// A `Vec<Rule>` containing extracted rules.
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
                protocol: format!("{}/{}", Self::map_ipprotocol(ipprotocol), protocol),
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

    /// Maps an IP protocol string to a common format.
    ///
    /// # Arguments
    /// * `ipprotocol` - A string representing the IP protocol.
    ///
    /// # Returns
    /// A `String` representing the mapped protocol (IPv4, IPv6, or Unknown).
    #[inline]
    fn map_ipprotocol(ipprotocol: &str) -> &str {
        match ipprotocol {
            "inet" => "IPv4",
            "inet6" => "IPv6",
            _ => "none",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PfSenseRulesParser;
    use roxmltree::Document;

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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let rules = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].disabled, false);
        assert_eq!(rules[0].r#type, "filter");
        assert_eq!(rules[0].policy, "pass");
        assert_eq!(rules[0].protocol, "IPv4/any");
        assert_eq!(rules[0].description, "Default allow LAN to any rule");
        assert_eq!(rules[0].source_addr, "lan");
        assert_eq!(rules[0].source_port, "*");
        assert_eq!(rules[0].source_type, "network");
        assert_eq!(rules[0].source_inversed, false);
        assert_eq!(rules[0].destination_addr, "*");
        assert_eq!(rules[0].destination_port, "*");
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let rules = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].disabled, false);
        assert_eq!(rules[0].r#type, "nat");
        assert_eq!(rules[0].policy, "pass");
        assert_eq!(rules[0].protocol, "IPv6/tcp");
        assert_eq!(rules[0].description, "NAT Rule");
        assert_eq!(rules[0].source_addr, "*");
        assert_eq!(rules[0].source_port, "*");
        assert_eq!(rules[0].source_type, "address");
        assert_eq!(rules[0].source_inversed, false);
        assert_eq!(rules[0].destination_addr, "wanip");
        assert_eq!(rules[0].destination_port, "8091");
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let rules = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 2);

        // Verify the first rule (Filter)
        assert_eq!(rules[0].disabled, true);
        assert_eq!(rules[0].r#type, "filter");
        assert_eq!(rules[0].policy, "pass");
        assert_eq!(rules[0].protocol, "IPv4/any");
        assert_eq!(rules[0].description, "Allow LAN");
        assert_eq!(rules[0].source_addr, "lan");
        assert_eq!(rules[0].source_port, "*");
        assert_eq!(rules[0].source_type, "network");
        assert_eq!(rules[0].source_inversed, false);
        assert_eq!(rules[0].destination_addr, "*");
        assert_eq!(rules[0].destination_port, "*");
        assert_eq!(rules[0].destination_type, "address");
        assert_eq!(rules[0].destination_inversed, false);
        assert_eq!(rules[0].interface, "lan");
        assert_eq!(rules[0].order, 0);

        // Verify the second rule (NAT)
        assert_eq!(rules[1].disabled, true);
        assert_eq!(rules[1].r#type, "nat");
        assert_eq!(rules[1].policy, "pass");
        assert_eq!(rules[1].protocol, "IPv4/tcp");
        assert_eq!(rules[1].description, "NAT Rule");
        assert_eq!(rules[1].source_addr, "*");
        assert_eq!(rules[1].source_port, "*");
        assert_eq!(rules[1].destination_addr, "wanip");
        assert_eq!(rules[1].destination_port, "8091");
        assert_eq!(rules[1].destination_type, "network");
        assert_eq!(rules[1].destination_inversed, false);
        assert_eq!(rules[1].interface, "wan");
        assert_eq!(rules[1].order, 0);
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let rules = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].disabled, false);
        assert_eq!(rules[0].r#type, "filter");
        assert_eq!(rules[0].policy, "reject");
        assert_eq!(rules[0].protocol, "IPv4/any");
        assert_eq!(rules[0].description, "Block traffic");
        assert_eq!(rules[0].source_addr, "*");
        assert_eq!(rules[0].source_port, "*");
        assert_eq!(rules[0].source_type, "address");
        assert_eq!(rules[0].source_inversed, false);
        assert_eq!(rules[0].destination_addr, "restricted_zone");
        assert_eq!(rules[0].destination_port, "*");
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let rules = PfSenseRulesParser::parse(&doc);

        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].order, 0);
        assert_eq!(rules[1].order, 1);
        assert_eq!(rules[2].order, 2);
    }
}
