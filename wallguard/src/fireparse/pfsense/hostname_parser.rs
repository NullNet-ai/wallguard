use xmltree::Element;

pub struct PfSenseHostnameParser {}

impl PfSenseHostnameParser {
    pub fn parse(document: &Element) -> String {
        if let Some(system_node) = document.get_child("system") {
            let hostname = system_node.get_child("hostname").and_then(|c| c.get_text());

            let domain = system_node.get_child("domain").and_then(|c| c.get_text());

            match (hostname, domain) {
                (Some(h), Some(d)) => format!("{}.{}", h, d),
                (Some(h), None) => h.to_string(),
                (None, Some(d)) => d.to_string(),
                _ => "none".to_string(),
            }
        } else {
            "none".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xmltree::Element;

    fn parse_xml(xml: &str) -> Element {
        Element::parse(xml.as_bytes()).expect("Failed to parse XML")
    }

    #[test]
    fn test_parse_hostname() {
        let xml = r#"<pfsense>
                        <system>
                            <hostname>router</hostname>
                            <domain>local</domain>
                        </system>
                    </pfsense>"#;
        let doc = parse_xml(xml);
        let hostname = PfSenseHostnameParser::parse(&doc);
        assert_eq!(hostname, "router.local");
    }

    #[test]
    fn test_parse_hostname_missing_domain() {
        let xml = r#"<pfsense>
                        <system>
                            <hostname>router</hostname>
                        </system>
                    </pfsense>"#;
        let doc = parse_xml(xml);
        let hostname = PfSenseHostnameParser::parse(&doc);
        assert_eq!(hostname, "router");
    }

    #[test]
    fn test_parse_hostname_empty_xml() {
        let xml = "<pfsense></pfsense>";
        let doc = parse_xml(xml);
        let hostname = PfSenseHostnameParser::parse(&doc);
        assert_eq!(hostname, "none");
    }
}
