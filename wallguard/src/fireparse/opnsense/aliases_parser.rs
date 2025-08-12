use wallguard_common::protobuf::wallguard_models::Alias;
use xmltree::{Element, XMLNode};

pub struct OpnSenseAliasesParser;

impl OpnSenseAliasesParser {
    pub fn parse(document: &Element) -> Vec<Alias> {
        let mut aliases = vec![];

        if let Some(node) = document
            .get_child("OPNsense")
            .and_then(|el| el.get_child("Firewall"))
            .and_then(|el| el.get_child("Alias"))
            .and_then(|el| el.get_child("aliases"))
        {
            for (_, alias) in node
                .children
                .iter()
                .filter_map(|anode| match anode {
                    XMLNode::Element(e) if e.name == "alias" => Some(e),
                    _ => None,
                })
                .enumerate()
            {
                let name = alias
                    .get_child("name")
                    .and_then(|el| el.get_text())
                    .unwrap_or("none".into())
                    .to_string();

                let r#type = alias
                    .get_child("type")
                    .and_then(|el| el.get_text())
                    .unwrap_or("none".into())
                    .to_string();

                let value = alias
                    .get_child("content")
                    .and_then(|el| el.get_text())
                    .unwrap_or("none".into())
                    .to_string();

                let description = alias
                    .get_child("content")
                    .and_then(|el| el.get_text())
                    .unwrap_or("none".into())
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
    use xmltree::Element;

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

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
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

        let doc = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        let aliases = OpnSenseAliasesParser::parse(&doc);

        assert_eq!(aliases.len(), 0);
    }
}
