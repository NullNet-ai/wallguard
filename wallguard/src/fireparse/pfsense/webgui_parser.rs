use xmltree::Element;

pub struct PfSenseWebGuiParser {}
impl PfSenseWebGuiParser {
    pub fn parse(document: &Element, default: &str) -> String {
        document
            .get_child("system")
            .and_then(|system| system.get_child("webgui"))
            .and_then(|webgui| webgui.get_child("protocol"))
            .and_then(|protocol| protocol.get_text())
            .unwrap_or(default.into())
            .to_string()
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

        let doc = parse_xml(xml);
        let protocol = PfSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "https");
    }

    #[test]
    fn test_parse_gui_protocol_default() {
        let xml = r#"<pfsense>
            <webgui>
            </webgui>
        </pfsense>"#;

        let doc = parse_xml(xml);
        let protocol = PfSenseWebGuiParser::parse(&doc, "http");
        assert_eq!(protocol, "http");
    }
}
