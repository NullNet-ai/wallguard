use serde::Serialize;

#[derive(Debug, Serialize, Hash, Eq, PartialEq)]
pub struct TransportHeader {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_port: Option<u16>,
    pub protocol: Protocol,
}

impl TransportHeader {
    pub fn from_etherparse(transport: Option<etherparse::TransportHeader>) -> Option<Self> {
        match transport {
            Some(etherparse::TransportHeader::Tcp(h)) => {
                let source_port = h.source_port;
                let destination_port = h.destination_port;
                Some(Self {
                    source_port: Some(source_port),
                    destination_port: Some(destination_port),
                    protocol: Protocol::Tcp,
                })
            }
            Some(etherparse::TransportHeader::Udp(h)) => {
                let source_port = h.source_port;
                let destination_port = h.destination_port;
                Some(Self {
                    source_port: Some(source_port),
                    destination_port: Some(destination_port),
                    protocol: Protocol::Udp,
                })
            }
            Some(etherparse::TransportHeader::Icmpv4(_)) => Some(Self {
                source_port: None,
                destination_port: None,
                protocol: Protocol::IcmpV4,
            }),
            Some(etherparse::TransportHeader::Icmpv6(_)) => Some(Self {
                source_port: None,
                destination_port: None,
                protocol: Protocol::IcmpV6,
            }),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash)]
pub enum Protocol {
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(rename = "udp")]
    Udp,
    #[serde(rename = "icmpv4")]
    IcmpV4,
    #[serde(rename = "icmpv6")]
    IcmpV6,
}

impl From<Protocol> for String {
    fn from(value: Protocol) -> Self {
        match value {
            Protocol::Tcp => String::from("tcp"),
            Protocol::Udp => String::from("udp"),
            Protocol::IcmpV4 => String::from("icmpv4"),
            Protocol::IcmpV6 => String::from("icmpv6"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_header_with_ports_to_json() {
        let transport_header = TransportHeader {
            source_port: Some(443),
            destination_port: Some(50051),
            protocol: Protocol::Tcp,
        };

        let json = serde_json::to_string(&transport_header).unwrap();

        assert_eq!(
            json,
            r#"{"source_port":443,"destination_port":50051,"protocol":"tcp"}"#
        );
    }

    #[test]
    fn test_transport_header_without_ports_to_json() {
        let transport_header = TransportHeader {
            source_port: None,
            destination_port: None,
            protocol: Protocol::IcmpV4,
        };

        let json = serde_json::to_string(&transport_header).unwrap();

        assert_eq!(json, r#"{"protocol":"icmpv4"}"#);
    }

    #[test]
    fn test_ip_protocol_serialize() {
        assert_eq!(serde_json::to_string(&Protocol::Tcp).unwrap(), r#""tcp""#);
        assert_eq!(serde_json::to_string(&Protocol::Udp).unwrap(), r#""udp""#);
        assert_eq!(
            serde_json::to_string(&Protocol::IcmpV4).unwrap(),
            r#""icmpv4""#
        );
        assert_eq!(
            serde_json::to_string(&Protocol::IcmpV6).unwrap(),
            r#""icmpv6""#
        );
    }
}
