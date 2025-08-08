use crate::Alias;
use xmltree::Document;

pub struct OpnSenseAliasesParser;

impl OpnSenseAliasesParser {
    pub fn parse(document: &Document) -> Vec<Alias> {
        let mut aliases = vec![];

        if let Some(aliases_node) = document
            .descendants()
            .find(|e| e.has_tag_name("opnsense"))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("OPNsense")))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("Firewall")))
            .and_then(|e| e.children().find(|ce| ce.has_tag_name("Alias")))
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

                let content = alias
                    .children()
                    .find(|e| e.has_tag_name("content"))
                    .and_then(|e| e.text());

                let value = content.unwrap_or("None").to_string();

                let description = alias
                    .children()
                    .find(|e| e.has_tag_name("description"))
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
    use super::OpnSenseAliasesParser;
    use xmltree::Document;

    #[test]
    fn test_parse_aliase() {
        let xml = r#"
        <opnsense>
            <OPNsense>
                <Firewall>
                    <Alias>
                        <aliases>
                            <alias>
                                <enabled>1</enabled>
                                <name>NoProxy</name>
                                <type>host</type>
                                <path_expression/>
                                <proto/>
                                <interface/>
                                <counters/>
                                <updatefreq/>
                                <content>@@aliascontent@@</content>
                                <password/>
                                <username/>
                                <authtype/>
                                <categories/>
                                <description>NoProxy group</description>
                            </alias>
                        </aliases>
                    </Alias>
                </Firewall>
            </OPNsense>
        </opnsense>
        "#;

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = OpnSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "NoProxy");
        assert_eq!(aliases[0].r#type, "host");
        assert_eq!(aliases[0].value, "@@aliascontent@@");
        assert_eq!(aliases[0].description, "NoProxy group");
    }

    #[test]
    fn test_parse_empty_aliases() {
        let xml = r#"
        <opnsense>
            <OPNsense>
                <Firewall>
                    <Alias>
                        <aliases>
                        </aliases>
                    </Alias>
                </Firewall>
            </OPNsense>
        </opnsense>
        "#;

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let aliases = OpnSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 0);
    }
}
