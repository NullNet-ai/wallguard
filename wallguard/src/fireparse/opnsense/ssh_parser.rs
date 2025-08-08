use crate::SSHConfig;
use roxmltree::Document;

const DEFAULT_PORT: u16 = 22;
const DEFAUKT_ENABLE: bool = false;
pub struct OpnSenseSSHParser {}

impl OpnSenseSSHParser {
    pub fn parse(document: &Document) -> SSHConfig {
        let ssh_node = document
            .descendants()
            .find(|e| e.has_tag_name("opnsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("system")))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("ssh")));

        if let Some(ssh_node) = ssh_node {
            let enabled = ssh_node.children().any(|e| e.has_tag_name("enable"));

            let port = ssh_node
                .children()
                .find(|e| e.has_tag_name("port"))
                .and_then(|e| e.text())
                .and_then(|t| t.parse::<u16>().ok())
                .unwrap_or(DEFAULT_PORT);

            SSHConfig { enabled, port }
        } else {
            SSHConfig {
                enabled: DEFAUKT_ENABLE,
                port: DEFAULT_PORT,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;

    const DEFAULT_PORT: u16 = 22;

    #[test]
    fn test_missing_all_nodes() {
        let xml = r#"<config></config>"#;
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SSHConfig {
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
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SSHConfig {
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
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SSHConfig {
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
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SSHConfig {
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
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SSHConfig {
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
        let doc = Document::parse(xml).expect("Failed to parse XML");
        let config = OpnSenseSSHParser::parse(&doc);
        assert_eq!(
            config,
            SSHConfig {
                enabled: false,
                port: 2222
            }
        );
    }
}