use roxmltree::Document;

/// A parser for extracting the hostname from a pfSense XML configuration.
pub struct PfSenseHostnameParser {}

impl PfSenseHostnameParser {
    /// Parses a given XML document and extracts the hostname and domain details.
    ///
    /// # Arguments
    ///
    /// * `document` - A reference to a `roxmltree::Document` containing pfSense configuration.
    ///
    /// # Returns
    ///
    /// A `String` representing the full hostname with domain if available.
    pub fn parse(document: &Document) -> String {
        if let Some(system_node) = document
            .descendants()
            .find(|e| e.has_tag_name("pfsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("system")))
        {
            let hostname = system_node
                .children()
                .find(|c| c.has_tag_name("hostname"))
                .and_then(|c| c.text());

            let domain = system_node
                .children()
                .find(|c| c.has_tag_name("domain"))
                .and_then(|c| c.text());

            if hostname.is_some() && domain.is_some() {
                return format!("{}.{}", hostname.unwrap(), domain.unwrap());
            } else if hostname.is_some() {
                return hostname.unwrap().to_string();
            } else {
                return domain.unwrap().to_string();
            }
        }

        String::from("none")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;

    #[test]
    fn test_parse_hostname() {
        let xml = r#"<pfsense>
                        <system>
                            <hostname>router</hostname>
                            <domain>local</domain>
                        </system>
                    </pfsense>"#;
        let doc = Document::parse(xml).expect("Failed to parse XML");
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
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let hostname = PfSenseHostnameParser::parse(&doc);
        assert_eq!(hostname, "router");
    }

    #[test]
    fn test_parse_hostname_empty_xml() {
        let xml = "<pfsense></pfsense>";
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let hostname = PfSenseHostnameParser::parse(&doc);
        assert_eq!(hostname, "none");
    }
}
