use crate::fireparse::Alias;
use roxmltree::Document;

/// A parser for extracting alias definitions from a pfSense XML configuration.
pub struct AliasesParser {}

impl AliasesParser {
    /// Parses a pfSense XML document to extract alias definitions.
    ///
    /// # Arguments
    /// * `document` - A reference to a `Document` containing the pfSense configuration.
    ///
    /// # Returns
    /// A `Vec<Alias>` containing all parsed aliases from the `<aliases>` section.
    pub fn parse(document: &Document) -> Vec<Alias> {
        let mut aliases = vec![];

        if let Some(aliases_node) = document
            .descendants()
            .find(|e| e.has_tag_name("pfsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("aliases")))
        {
            for alias in aliases_node.children().filter(|e| e.has_tag_name("alias")) {
                let name = alias
                    .children()
                    .find(|e| e.has_tag_name("name"))
                    .and_then(|e| e.text())
                    .unwrap_or("none")
                    .to_string();

                let r#type = alias
                    .children()
                    .find(|e| e.has_tag_name("type"))
                    .and_then(|e| e.text())
                    .unwrap_or("none")
                    .to_string();

                let address = alias
                    .children()
                    .find(|e| e.has_tag_name("address"))
                    .and_then(|e| e.text());

                let url = alias
                    .children()
                    .find(|e| e.has_tag_name("url"))
                    .and_then(|e| e.text());

                let value = address.or(url).unwrap_or("None").to_string();

                let description = alias
                    .children()
                    .find(|e| e.has_tag_name("descr"))
                    .and_then(|e| e.text())
                    .unwrap_or("")
                    .to_string();

                aliases.push(Alias {
                    r#type,
                    name,
                    value,
                    description,
                });
            }
        }
        aliases
    }
}

#[cfg(test)]
mod tests {
    use super::AliasesParser;
    use roxmltree::Document;

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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = AliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 2);

        assert_eq!(aliases[0].name, "Ports");
        assert_eq!(aliases[0].r#type, "port");
        assert_eq!(aliases[0].value, "1 2 3");
        assert_eq!(aliases[0].description, "Description");

        assert_eq!(aliases[1].name, "Addresses");
        assert_eq!(aliases[1].r#type, "host");
        assert_eq!(aliases[1].value, "1.1.1.1 1.1.1.2 1.1.1.3");
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = AliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "Networks");
        assert_eq!(aliases[0].r#type, "network");
        assert_eq!(aliases[0].value, "1.1.1.0/24 2.2.2.0/24");
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = AliasesParser::parse(&doc);

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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = AliasesParser::parse(&doc);

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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = AliasesParser::parse(&doc);

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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = AliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 0);
    }
}
