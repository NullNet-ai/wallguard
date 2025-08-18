use xmltree::Element;

pub struct OpnSenseWebGuiParser {}

impl OpnSenseWebGuiParser {
    pub fn parse(root: &Element, default: &str) -> String {
        root.get_child("system")
            .and_then(|e| e.get_child("webgui"))
            .and_then(|e| e.get_child("protocol"))
            .and_then(|e| e.get_text())
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xmltree::Element;

    #[test]
    fn test_parse_gui_protocol() {
        let xml = r#"<opnsense>
            <system>
                <webgui>
                    <protocol>https</protocol>
                    <ssl-certref>598edde7a20b2</ssl-certref>
                    <port/>
                    <ssl-ciphers/>
                    <compression/>
                </webgui>
            </system>
        </opnsense>"#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let protocol = OpnSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "https");
    }

    #[test]
    fn test_parse_gui_protocol_default() {
        let xml = r#"<opnsense>
            <webgui>

            </webgui>
        </opnsense>"#;

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let protocol = OpnSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "http");
    }
}
