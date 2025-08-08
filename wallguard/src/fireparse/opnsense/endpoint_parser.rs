use xmltree::Node;

const ANY_ADDR_VALUE: &str = "*";
const ANY_PORT_VALUE: &str = "*";
const DEFAULT_TYPE_VALUE: &str = "address";
const DEFAULT_INVERSED: bool = false;

pub struct EndpointParser {}

impl EndpointParser {
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

    fn parse_port(node: &Node) -> String {
        node.children()
            .find(|e| e.has_tag_name("port"))
            .and_then(|e| e.text())
            .unwrap_or(ANY_PORT_VALUE)
            .to_string()
    }

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

    fn parse_addr_type(node: &Node) -> String {
        match node.children().any(|e| e.has_tag_name("network")) {
            true => String::from("network"),
            false => String::from("address"),
        }
    }

    fn parse_inversed(node: &Node) -> bool {
        node.children().any(|e| e.has_tag_name("not"))
    }
}
