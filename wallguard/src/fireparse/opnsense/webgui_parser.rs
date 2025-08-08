use roxmltree::Document;

pub struct OpnSenseWebGuiParser {}

impl OpnSenseWebGuiParser {
    pub fn parse(document: &Document, default: &str) -> String {
        document
            .descendants()
            .find(|e| e.has_tag_name("opnsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("system")))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("webgui")))
            .and_then(|wn| wn.children().find(|ch| ch.has_tag_name("protocol")))
            .and_then(|pn| pn.text())
            .unwrap_or(default)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;

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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let protocol = OpnSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "https");
    }

    #[test]
    fn test_parse_gui_protocol_default() {
        let xml = r#"<opnsense>
            <webgui>

            </webgui>
        </opnsense>"#;

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let protocol = OpnSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "http");
    }
}
