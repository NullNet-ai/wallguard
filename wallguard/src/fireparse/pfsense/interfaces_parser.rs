use wallguard_common::os_if::InterfaceSnapshot;
use wallguard_common::protobuf::wallguard_models::{IpAddress, NetworkInterface};
use xmltree::Element;

pub struct PfSenseInterfacesParser {}

impl PfSenseInterfacesParser {
    pub fn parse(
        document: &Element,
        os_interfaces: Vec<InterfaceSnapshot>,
    ) -> Vec<NetworkInterface> {
        let mut interfaces = vec![];

        if let Some(interfaces_node) = document.get_child("interfaces") {
            for interface in interfaces_node
                .children
                .iter()
                .filter_map(|n| n.as_element())
            {
                let name = interface.name.clone();

                let device = interface
                    .get_child("if")
                    .and_then(|c| c.get_text())
                    .unwrap_or("none".into())
                    .to_string();

                let mut addresses = vec![];

                if let Some(data) = os_interfaces.iter().find(|iface| iface.name == device) {
                    addresses = data
                        .ip_addresses
                        .iter()
                        .map(|addr| IpAddress {
                            address: addr.to_string(),
                            version: if addr.is_ipv4() { 4 } else { 6 },
                        })
                        .collect();
                }

                if addresses.is_empty() {
                    if let Some(ipv4) = interface.get_child("ipaddr").and_then(|c| c.get_text()) {
                        addresses.push(IpAddress {
                            address: ipv4.to_string(),
                            version: 4,
                        });
                    }

                    if let Some(ipv6) = interface.get_child("ipaddrv6").and_then(|c| c.get_text()) {
                        addresses.push(IpAddress {
                            address: ipv6.to_string(),
                            version: 6,
                        });
                    }
                }

                interfaces.push(NetworkInterface {
                    name,
                    device,
                    addresses,
                });
            }
        }

        interfaces
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use wallguard_common::os_if::InterfaceSnapshot;
    use xmltree::Element;

    fn parse_xml(xml: &str) -> Element {
        Element::parse(xml.as_bytes()).expect("Failed to parse XML")
    }

    #[test]
    fn test_parse_valid_xml() {
        let xml = r#"<pfsense>
                        <interfaces>
                            <wan>
                                <if>igb0</if>
                                <ipaddr>192.168.1.1</ipaddr>
                            </wan>
                            <lan>
                                <if>igb1</if>
                                <ipaddr>192.168.1.2</ipaddr>
                            </lan>
                        </interfaces>
                    </pfsense>"#;

        let doc = parse_xml(xml);
        let interfaces = PfSenseInterfacesParser::parse(&doc, vec![]);

        assert_eq!(interfaces.len(), 2);
        assert_eq!(interfaces[0].name, "wan");
        assert_eq!(interfaces[0].device, "igb0");
        assert_eq!(interfaces[0].addresses.len(), 1);
        assert_eq!(interfaces[0].addresses[0].address, "192.168.1.1");
        assert_eq!(interfaces[0].addresses[0].version, 4);

        assert_eq!(interfaces[1].name, "lan");
        assert_eq!(interfaces[1].device, "igb1");
        assert_eq!(interfaces[1].addresses.len(), 1);
        assert_eq!(interfaces[1].addresses[0].address, "192.168.1.2");
        assert_eq!(interfaces[1].addresses[0].version, 4);
    }

    #[test]
    fn test_parse_missing_elements() {
        let xml = r#"<pfsense>
                        <interfaces>
                            <wan></wan>
                        </interfaces>
                    </pfsense>"#;

        let doc = parse_xml(xml);
        let interfaces = PfSenseInterfacesParser::parse(&doc, vec![]);

        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name, "wan");
        assert_eq!(interfaces[0].device, "none");
        assert_eq!(interfaces[0].addresses.len(), 0);
    }

    #[test]
    fn test_parse_empty_xml() {
        let xml = "<pfsense></pfsense>";
        let doc = parse_xml(xml);
        let interfaces = PfSenseInterfacesParser::parse(&doc, vec![]);

        assert_eq!(interfaces.len(), 0);
    }

    #[test]
    fn test_ifaces_data_override_xml_contents() {
        let xml = r#"
        <pfsense>
            <interfaces>
                <wan>
                    <if>igb0</if>
                    <ipaddr>192.168.1.1</ipaddr>
                </wan>
            </interfaces>
        </pfsense>"#;

        let doc = parse_xml(xml);

        let iface_data = InterfaceSnapshot {
            name: "igb0".to_string(),
            is_up: true,
            is_loopback: false,
            is_multicast: true,
            is_broadcast: true,
            mac_address: None,
            interface_index: None,
            ip_addresses: vec![
                "8.8.8.8".parse::<IpAddr>().unwrap(),
                "8.8.4.4".parse::<IpAddr>().unwrap(),
            ],
            subnet_mask: None,
            gateway: None,
        };

        let interfaces = PfSenseInterfacesParser::parse(&doc, vec![iface_data]);

        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name, "wan");
        assert_eq!(interfaces[0].device, "igb0");
        assert_eq!(interfaces[0].addresses.len(), 2);
        assert_eq!(interfaces[0].addresses[0].address, "8.8.8.8");
        assert_eq!(interfaces[0].addresses[0].version, 4);
        assert_eq!(interfaces[0].addresses[1].address, "8.8.4.4");
        assert_eq!(interfaces[0].addresses[1].version, 4);
    }
}
