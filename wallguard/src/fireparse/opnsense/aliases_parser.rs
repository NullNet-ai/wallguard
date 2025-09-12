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
            for alias in node.children.iter().filter_map(|anode| match anode {
                XMLNode::Element(e) if e.name == "alias" => Some(e),
                _ => None,
            }) {
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

                let url = alias.get_child("url").and_then(|e| e.get_text());

                let value = alias
                    .get_child("content")
                    .and_then(|e| e.get_text())
                    .or(url)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "None".to_string())
                    .split("\n")
                    .collect::<Vec<&str>>()
                    .join(",");

                let description = alias
                    .get_child("description")
                    .and_then(|el| el.get_text())
                    .unwrap_or("none".into())
                    .to_string();

                aliases.push(Alias {
                    r#type,
                    name,
                    value,
                    description,
                    ..Default::default()
                });
            }
        }

        aliases
    }

    pub fn to_element(alias: Alias) -> Element {
        let mut alias_elem = Element::new("alias");

        let mut name_elem = Element::new("name");
        name_elem.children.push(XMLNode::Text(alias.name));
        alias_elem.children.push(XMLNode::Element(name_elem));

        let mut type_elem = Element::new("type");
        type_elem.children.push(XMLNode::Text(alias.r#type));
        alias_elem.children.push(XMLNode::Element(type_elem));

        let mut content_elem = Element::new("content");
        let value = alias.value.split(",").collect::<Vec<&str>>().join("\n");
        content_elem.children.push(XMLNode::Text(value));
        alias_elem.children.push(XMLNode::Element(content_elem));

        let mut description_elem = Element::new("description");
        description_elem
            .children
            .push(XMLNode::Text(alias.description));
        alias_elem.children.push(XMLNode::Element(description_elem));

        alias_elem
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
                                <content>1.1.1.1
2.2.2.2</content>
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
        assert_eq!(aliases[0].value, "1.1.1.1,2.2.2.2");
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
