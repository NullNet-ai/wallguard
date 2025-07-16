use roxmltree::Document;

pub struct PfSenseWebGuiParser {}

impl PfSenseWebGuiParser {
    pub fn parse(document: &Document, default: &str) -> String {
        document
            .descendants()
            .find(|e| e.has_tag_name("pfsense"))
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
        let xml = r#"<pfsense>
            <system>
                <webgui>
                    <protocol>https</protocol>
                    <loginautocomplete/>
                    <ssl-certref>665b4a5edc246</ssl-certref>
                    <dashboardcolumns>2</dashboardcolumns>
                    <max_procs>2</max_procs>
                    <roaming>enabled</roaming>
                    <webguicss>pfSense.css</webguicss>
                    <logincss>1e3f75;</logincss>
                </webgui>
            </system>
        </pfsense>"#;

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let protocol = PfSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "https");
    }

    #[test]
    fn test_parse_gui_protocol_default() {
        let xml = r#"<pfsense>
            <webgui>

            </webgui>
        </pfsense>"#;

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let protocol = PfSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "http");
    }
}
