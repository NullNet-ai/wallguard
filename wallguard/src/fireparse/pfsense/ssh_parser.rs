use wallguard_common::protobuf::wallguard_models::SshConfig;

use xmltree::Element;

const DEFAULT_PORT: u32 = 22;
const DEFAULT_ENABLE: bool = false;
pub struct PfSenseSSHParser {}

impl PfSenseSSHParser {
    pub fn parse(document: &Element) -> SshConfig {
        let ssh_node = document
            .get_child("system")
            .and_then(|sys| sys.get_child("ssh"));

        if let Some(ssh_node) = ssh_node {
            let enabled = ssh_node
                .children
                .iter()
                .filter_map(|c| c.as_element())
                .any(|e| e.name == "enable");

            let port = ssh_node
                .get_child("port")
                .and_then(|e| e.get_text())
                .and_then(|text| text.parse::<u32>().ok())
                .unwrap_or(DEFAULT_PORT);

            SshConfig { enabled, port }
        } else {
            SshConfig {
                enabled: DEFAULT_ENABLE,
                port: DEFAULT_PORT,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wallguard_common::protobuf::wallguard_models::SshConfig;
    use xmltree::Element;

    const DEFAULT_PORT: u32 = 22;

    fn parse_xml(xml: &str) -> Element {
        Element::parse(xml.as_bytes()).expect("Failed to parse XML")
    }

    #[test]
    fn test_missing_all_nodes() {
        let xml = r#"<config></config>"#;
        let doc = parse_xml(xml);
        let config = PfSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SshConfig {
                enabled: false,
                port: DEFAULT_PORT
            }
        );
    }

    #[test]
    fn test_missing_ssh_node() {
        let xml = r#"
            <pfsense>
                <system>
                    <other>value</other>
                </system>
            </pfsense>
        "#;
        let doc = parse_xml(xml);
        let config = PfSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SshConfig {
                enabled: false,
                port: DEFAULT_PORT
            }
        );
    }

    #[test]
    fn test_enabled_with_default_port() {
        let xml = r#"
            <pfsense>
                <system>
                    <ssh>
                        <enable/>
                    </ssh>
                </system>
            </pfsense>
        "#;
        let doc = parse_xml(xml);
        let config = PfSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SshConfig {
                enabled: true,
                port: DEFAULT_PORT
            }
        );
    }

    #[test]
    fn test_enabled_with_custom_port() {
        let xml = r#"
            <pfsense>
                <system>
                    <ssh>
                        <enable/>
                        <port>2222</port>
                    </ssh>
                </system>
            </pfsense>
        "#;
        let doc = parse_xml(xml);
        let config = PfSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SshConfig {
                enabled: true,
                port: 2222
            }
        );
    }

    #[test]
    fn test_invalid_port_falls_back_to_default() {
        let xml = r#"
            <pfsense>
                <system>
                    <ssh>
                        <enable/>
                        <port>not_a_number</port>
                    </ssh>
                </system>
            </pfsense>
        "#;
        let doc = parse_xml(xml);
        let config = PfSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SshConfig {
                enabled: true,
                port: DEFAULT_PORT
            }
        );
    }

    #[test]
    fn test_disabled_no_enable_tag() {
        let xml = r#"
            <pfsense>
                <system>
                    <ssh>
                        <port>2222</port>
                    </ssh>
                </system>
            </pfsense>
        "#;
        let doc = parse_xml(xml);
        let config = PfSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SshConfig {
                enabled: false,
                port: 2222
            }
        );
    }
}
