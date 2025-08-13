use xmltree::Element;

pub struct OpnSenseHostnameParser {}

impl OpnSenseHostnameParser {
    pub fn parse(document: &Element) -> String {
        if let Some(system_node) = document.get_child("system") {
            let hostname = system_node.get_child("hostname").and_then(|c| c.get_text());

            let domain = system_node.get_child("domain").and_then(|c| c.get_text());

            match (hostname, domain) {
                (Some(h), Some(d)) => format!("{h}.{d}"),
                (Some(h), None) => h.to_string(),
                (None, Some(d)) => d.to_string(),
                _ => "none".to_string(),
            }
        } else {
            "none".to_string()
        }
    }
}
