use wallguard_common::protobuf::wallguard_models::SshConfig;
use xmltree::{Element, XMLNode};

const DEFAULT_PORT: u32 = 22;
const DEFAULT_ENABLE: bool = false;
pub struct OpnSenseSSHParser {}

impl OpnSenseSSHParser {
    pub fn parse(root: &Element) -> SshConfig {
        let ssh_node = root.get_child("system").and_then(|e| e.get_child("ssh"));

        if let Some(ssh) = ssh_node {
            let enabled = ssh
                .children
                .iter()
                .any(|node| matches!(node, XMLNode::Element(e) if e.name == "enable"));

            let port = ssh
                .get_child("port")
                .and_then(|e| e.get_text())
                .and_then(|t| t.parse::<u32>().ok())
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
    use xmltree::Element;

    const DEFAULT_PORT: u32 = 22;

    #[test]
    fn test_missing_all_nodes() {
        let xml = r#"<config></config>"#;
        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
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
            <opnsense>
                <system>
                    <other>value</other>
                </system>
            </opnsense>
        "#;
        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
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
            <opnsense>
                <system>
                    <ssh>
                        <enable/>
                    </ssh>
                </system>
            </opnsense>
        "#;
        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
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
            <opnsense>
                <system>
                    <ssh>
                        <enable/>
                        <port>2222</port>
                    </ssh>
                </system>
            </opnsense>
        "#;
        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
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
            <opnsense>
                <system>
                    <ssh>
                        <enable/>
                        <port>not_a_number</port>
                    </ssh>
                </system>
            </opnsense>
        "#;
        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
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
            <opnsense>
                <system>
                    <ssh>
                        <port>2222</port>
                    </ssh>
                </system>
            </opnsense>
        "#;
        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SshConfig {
                enabled: false,
                port: 2222
            }
        );
    }
}
