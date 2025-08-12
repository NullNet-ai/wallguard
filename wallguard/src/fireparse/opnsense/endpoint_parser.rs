use xmltree::Element;

const ANY_ADDR_VALUE: &str = "*";
const ANY_PORT_VALUE: &str = "*";
const DEFAULT_TYPE_VALUE: &str = "address";
const DEFAULT_INVERSED: bool = false;

pub struct EndpointParser {}

impl EndpointParser {
    pub fn parse(element: Option<&Element>) -> (String, String, String, bool) {
        if let Some(element_value) = element {
            let addr = Self::parse_addr(&element_value);
            let port = Self::parse_port(&element_value);
            let r#type = Self::parse_addr_type(&element_value);
            let inversed = Self::parse_inversed(&element_value);

            (addr, port, r#type, inversed)
        } else {
            (
                String::from(ANY_ADDR_VALUE),
                String::from(ANY_PORT_VALUE),
                String::from(DEFAULT_TYPE_VALUE),
                DEFAULT_INVERSED,
            )
        }
    }

    fn parse_port(node: &Element) -> String {
        node.get_child("port")
            .and_then(|e| e.get_text())
            .map(|s| s.to_string())
            .unwrap_or_else(|| ANY_PORT_VALUE.to_string())
    }

    fn parse_addr(node: &Element) -> String {
        if node.get_child("any").is_some() {
            return ANY_ADDR_VALUE.to_string();
        }

        if let Some(address) = node.get_child("address").and_then(|e| e.get_text()) {
            return address.to_string();
        }

        if let Some(network) = node.get_child("network").and_then(|e| e.get_text()) {
            return network.to_string();
        }

        ANY_ADDR_VALUE.to_string()
    }

    fn parse_addr_type(node: &Element) -> String {
        if node.get_child("network").is_some() {
            "network".to_string()
        } else {
            "address".to_string()
        }
    }

    fn parse_inversed(node: &Element) -> bool {
        node.get_child("not").is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::EndpointParser;
    use xmltree::Element;

    fn get_destination(xml: &str) -> Option<Element> {
        let root = Element::parse(xml.as_bytes()).expect("Failed to parse XML");
        root.get_child("destination").cloned()
    }

    #[test]
    fn test_parse_destination_with_address_and_port() {
        let xml = r#"
        <root>
            <destination>
                <address>1.1.1.1</address>
                <port>8080</port>
            </destination>
        </root>
        "#;

        let node = get_destination(xml);
        let (addr, port, r#type, inversed) = EndpointParser::parse(node.as_ref());
        assert_eq!(addr, "1.1.1.1");
        assert_eq!(port, "8080");
        assert_eq!(r#type, "address");
        assert_eq!(inversed, false);
    }

    #[test]
    fn test_parse_destination_with_network() {
        let xml = r#"
        <root>
            <destination>
                <network>wanip</network>
            </destination>
        </root>
        "#;

        let node = get_destination(xml);
        let (addr, port, r#type, inversed) = EndpointParser::parse(node.as_ref());
        assert_eq!(addr, "wanip");
        assert_eq!(port, "*");
        assert_eq!(r#type, "network");
        assert_eq!(inversed, false);
    }

    #[test]
    fn test_parse_destination_with_any() {
        let xml = r#"
        <root>
            <destination>
                <any></any>
                <port>123</port>
            </destination>
        </root>
        "#;

        let node = get_destination(xml);
        let (addr, port, r#type, inversed) = EndpointParser::parse(node.as_ref());
        assert_eq!(addr, "*");
        assert_eq!(port, "123");
        assert_eq!(r#type, "address");
        assert_eq!(inversed, false);
    }

    #[test]
    fn test_parse_destination_with_inversed() {
        let xml = r#"
        <root>
            <destination>
                <address>3.3.3.3</address>
                <not></not>
            </destination>
        </root>
        "#;

        let node = get_destination(xml);
        let (addr, port, r#type, inversed) = EndpointParser::parse(node.as_ref());
        assert_eq!(addr, "3.3.3.3");
        assert_eq!(port, "*");
        assert_eq!(r#type, "address");
        assert_eq!(inversed, true);
    }

    #[test]
    fn test_no_node_returns_defaults() {
        let (addr, port, r#type, inversed) = EndpointParser::parse(None);
        assert_eq!(addr, "*");
        assert_eq!(port, "*");
        assert_eq!(r#type, "address");
        assert_eq!(inversed, false);
    }
}
