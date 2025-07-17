use roxmltree::Node;

const ANY_ADDR_VALUE: &str = "*";
const ANY_PORT_VALUE: &str = "*";
const DEFAULT_TYPE_VALUE: &str = "address";
const DEFAULT_INVERSED: bool = false;

/// A parser for extracting endpoint information from `source` and `destination` nodes.
pub struct EndpointParser {}

impl EndpointParser {
    /// Parses a `source` or `destination` node to extract the address, port, type, and inversion state.
    ///
    /// # Arguments
    /// * `node` - An optional `Node` representing an endpoint.
    ///
    /// # Returns
    /// A tuple `(String, String, String, bool)` where:
    /// - The first element is the address, extracted from the `<address>` or `<network>` tag, or `"*"` if missing.
    /// - The second element is the port, extracted from the `<port>` tag, or `"*"` if missing.
    /// - The third element is the type, determined based on whether a `<network>` tag is present (`"network"` or `"address"`).
    /// - The fourth element is a boolean indicating whether the node contains a `<not>` tag (`true` for inversed).
    pub fn parse(node: Option<Node>) -> (String, String, String, bool) {
        if node.is_none() {
            return (
                String::from(ANY_ADDR_VALUE),
                String::from(ANY_PORT_VALUE),
                String::from(DEFAULT_TYPE_VALUE),
                DEFAULT_INVERSED,
            );
        }

        let node_value = node.unwrap();

        let addr = Self::parse_addr(&node_value);
        let port = Self::parse_port(&node_value);
        let r#type = Self::parse_addr_type(&node_value);
        let inversed = Self::parse_inversed(&node_value);

        (addr, port, r#type, inversed)
    }

    /// Extracts the port from a `source` or `destination` node.
    ///
    /// # Arguments
    /// * `node` - A reference to an XML node.
    ///
    /// # Returns
    /// A `String` containing the port value extracted from the `<port>` tag.
    /// If the `<port>` tag is missing, it defaults to `ANY_PORT_VALUE`.
    fn parse_port(node: &Node) -> String {
        node.children()
            .find(|e| e.has_tag_name("port"))
            .and_then(|e| e.text())
            .unwrap_or(ANY_PORT_VALUE)
            .to_string()
    }

    /// Extracts the address from a `source` or `destination` node.
    ///
    /// # Arguments
    /// * `node` - A reference to an XML node.
    ///
    /// # Returns
    /// A `String` containing:
    /// - `ANY_ADDR_VALUE` if `<any>` tag is present.
    /// - The value from the `<address>` tag, if present.
    /// - The value from the `<network>` tag, if `<address>` is missing.
    /// - `ANY_ADDR_VALUE` if neither tag is found.
    fn parse_addr(node: &Node) -> String {
        if node.children().any(|e| e.has_tag_name("any")) {
            return String::from(ANY_ADDR_VALUE);
        }

        if let Some(address) = node
            .children()
            .find(|e| e.has_tag_name("address"))
            .and_then(|e| e.text())
        {
            return String::from(address);
        }

        if let Some(network) = node
            .children()
            .find(|e| e.has_tag_name("network"))
            .and_then(|e| e.text())
        {
            return String::from(network);
        }

        String::from(ANY_ADDR_VALUE)
    }

    /// Determines the type of the endpoint based on whether a `<network>` tag is present.
    ///
    /// # Arguments
    /// * `node` - A reference to an XML node.
    ///
    /// # Returns
    /// A `String` representing the type:
    /// - `"network"` if the node contains a `<network>` tag.
    /// - `"address"` otherwise.
    fn parse_addr_type(node: &Node) -> String {
        match node.children().any(|e| e.has_tag_name("network")) {
            true => String::from("network"),
            false => String::from("address"),
        }
    }

    /// Checks if an endpoint node is inversed (negated).
    ///
    /// # Arguments
    /// * `node` - A reference to an XML node.
    ///
    /// # Returns
    /// `true` if the `<not>` tag is present, indicating inversion.
    /// Otherwise, returns `false`.
    fn parse_inversed(node: &Node) -> bool {
        node.children().any(|e| e.has_tag_name("not"))
    }
}

#[cfg(test)]
mod tests {
    use super::EndpointParser;
    use roxmltree::Document;

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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let node = doc
            .descendants()
            .find(|n| n.has_tag_name("destination"))
            .unwrap();

        let (addr, port, r#type, inversed) = EndpointParser::parse(Some(node));
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let node = doc
            .descendants()
            .find(|n| n.has_tag_name("destination"))
            .unwrap();

        let (addr, port, r#type, inversed) = EndpointParser::parse(Some(node));
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let node = doc
            .descendants()
            .find(|n| n.has_tag_name("destination"))
            .unwrap();

        let (addr, port, r#type, inversed) = EndpointParser::parse(Some(node));
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

        let doc = Document::parse(xml).expect("Failed to parse XML");
        let node = doc
            .descendants()
            .find(|n| n.has_tag_name("destination"))
            .unwrap();

        let (addr, port, r#type, inversed) = EndpointParser::parse(Some(node));
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
