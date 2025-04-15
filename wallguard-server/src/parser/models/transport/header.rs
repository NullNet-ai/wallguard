use serde::Serialize;

#[derive(Debug, Serialize, Default, Hash, Eq, PartialEq)]
pub struct TransportHeader {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_port: Option<u16>,
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
                })
            }
            Some(etherparse::TransportHeader::Udp(h)) => {
                let source_port = h.source_port;
                let destination_port = h.destination_port;
                Some(Self {
                    source_port: Some(source_port),
                    destination_port: Some(destination_port),
                })
            }
            Some(
                etherparse::TransportHeader::Icmpv4(_) | etherparse::TransportHeader::Icmpv6(_),
            ) => Some(Self::default()),
            None => None,
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
        };

        let json = serde_json::to_string(&transport_header).unwrap();

        assert_eq!(json, r#"{"source_port":443,"destination_port":50051}"#);
    }

    #[test]
    fn test_transport_header_without_ports_to_json() {
        let transport_header = TransportHeader {
            source_port: None,
            destination_port: None,
        };

        let json = serde_json::to_string(&transport_header).unwrap();

        assert_eq!(json, r#"{}"#);
    }
}
