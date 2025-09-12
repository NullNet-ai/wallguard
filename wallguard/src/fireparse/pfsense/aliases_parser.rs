use wallguard_common::protobuf::wallguard_models::Alias;
use xmltree::{Element, XMLNode};

pub struct PfSenseAliasesParser {}

impl PfSenseAliasesParser {
    pub fn parse(document: &Element) -> Vec<Alias> {
        let mut aliases = vec![];

        if let Some(aliases_node) = document.get_child("aliases") {
            for alias_node in aliases_node
                .children
                .iter()
                .filter_map(|node| node.as_element())
                .filter(|e| e.name == "alias")
            {
                let name = alias_node
                    .get_child("name")
                    .and_then(|e| e.get_text())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "none".to_string());

                let r#type = alias_node
                    .get_child("type")
                    .and_then(|e| e.get_text())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "none".to_string());

                let url = alias_node.get_child("url").and_then(|e| e.get_text());

                let value = alias_node
                    .get_child("address")
                    .and_then(|e| e.get_text())
                    .or(url)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "None".to_string())
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(",");

                let description = alias_node
                    .get_child("descr")
                    .and_then(|e| e.get_text())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                aliases.push(Alias {
                    r#type,
                    name,
                    value,
                    description,
                    ..Default::default()
                });
            }
        }

        aliases
    }

    pub fn to_element(alias: Alias) -> Element {
        let mut alias_elem = Element::new("alias");

        let mut name_elem = Element::new("name");

        name_elem.children.push(XMLNode::Text(alias.name));
        alias_elem.children.push(XMLNode::Element(name_elem));

        let mut type_elem = Element::new("type");
        type_elem.children.push(XMLNode::Text(alias.r#type));
        alias_elem.children.push(XMLNode::Element(type_elem));

        let mut content_elem = Element::new("content");
        let value = alias.value.split(",").collect::<Vec<&str>>().join(" ");
        content_elem.children.push(XMLNode::Text(value));
        alias_elem.children.push(XMLNode::Element(content_elem));

        let mut description_elem = Element::new("description");
        description_elem
            .children
            .push(XMLNode::Text(alias.description));
        alias_elem.children.push(XMLNode::Element(description_elem));

        alias_elem
    }
}

#[cfg(test)]
mod tests {
    use super::PfSenseAliasesParser;
    use xmltree::Element;

    #[test]
    fn test_parse_aliases() {
        let xml = r#"
        <pfsense>
            <aliases>
                <alias>
                    <name>Ports</name>
                    <type>port</type>
                    <address>1 2 3</address>
                    <descr><![CDATA[Description]]></descr>
                    <detail><![CDATA[Desc 1||Desc 2||Desc 3]]></detail>
                </alias>
                <alias>
                    <name>Addresses</name>
                    <type>host</type>
                    <address>1.1.1.1 1.1.1.2 1.1.1.3</address>
                    <descr><![CDATA[Description]]></descr>
                    <detail><![CDATA[Desc 1||Desc 2||Desc 3]]></detail>
                </alias>
            </aliases>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let aliases = PfSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 2);

        assert_eq!(aliases[0].name, "Ports");
        assert_eq!(aliases[0].r#type, "port");
        assert_eq!(aliases[0].value, "1,2,3");
        assert_eq!(aliases[0].description, "Description");

        assert_eq!(aliases[1].name, "Addresses");
        assert_eq!(aliases[1].r#type, "host");
        assert_eq!(aliases[1].value, "1.1.1.1,1.1.1.2,1.1.1.3");
        assert_eq!(aliases[1].description, "Description");
    }

    #[test]
    fn test_parse_alias_with_networks() {
        let xml = r#"
        <pfsense>
            <aliases>
                <alias>
                    <name>Networks</name>
                    <type>network</type>
                    <address>1.1.1.0/24 2.2.2.0/24</address>
                    <descr><![CDATA[Description]]></descr>
                    <detail><![CDATA[Network 1||Network 2]]></detail>
                </alias>
            </aliases>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let aliases = PfSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "Networks");
        assert_eq!(aliases[0].r#type, "network");
        assert_eq!(aliases[0].value, "1.1.1.0/24,2.2.2.0/24");
        assert_eq!(aliases[0].description, "Description");
    }

    #[test]
    fn test_parse_alias_with_port_range() {
        let xml = r#"
        <pfsense>
            <aliases>
                <alias>
                    <name>PortRange</name>
                    <type>port</type>
                    <address>1:10</address>
                    <descr><![CDATA[Description]]></descr>
                    <detail><![CDATA[Port range]]></detail>
                </alias>
            </aliases>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let aliases = PfSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "PortRange");
        assert_eq!(aliases[0].r#type, "port");
        assert_eq!(aliases[0].value, "1:10");
        assert_eq!(aliases[0].description, "Description");
    }

    #[test]
    fn test_parse_alias_with_url() {
        let xml = r#"
        <pfsense>
            <aliases>
                <alias>
                    <name>pfB_PRI1_v4</name>
                    <type>urltable</type>
                    <url>https://127.0.0.1:443/pfblockerng/pfblockerng.php?pfb=pfB_PRI1_v4</url>
                    <updatefreq>32</updatefreq>
                    <address></address>
                    <descr><![CDATA[pfBlockerNG  Auto  Alias [Abuse_Feodo_C2_v4]]]></descr>
                    <detail><![CDATA[DO NOT EDIT THIS ALIAS]]></detail>
                </alias>
            </aliases>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let aliases = PfSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "pfB_PRI1_v4");
        assert_eq!(aliases[0].r#type, "urltable");
        assert_eq!(
            aliases[0].value,
            "https://127.0.0.1:443/pfblockerng/pfblockerng.php?pfb=pfB_PRI1_v4"
        );
        assert_eq!(
            aliases[0].description,
            "pfBlockerNG  Auto  Alias [Abuse_Feodo_C2_v4]"
        );
    }

    #[test]
    fn test_parse_aliases_with_missing_fields() {
        let xml = r#"
        <pfsense>
            <aliases>
                <alias>
                    <name>MissingFields</name>
                </alias>
            </aliases>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let aliases = PfSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "MissingFields");
        assert_eq!(aliases[0].r#type, "none");
        assert_eq!(aliases[0].value, "None");
        assert_eq!(aliases[0].description, "");
    }

    #[test]
    fn test_parse_empty_aliases() {
        let xml = r#"
        <pfsense>
            <aliases></aliases>
        </pfsense>
        "#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let aliases = PfSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 0);
    }
}
