use crate::parser::models::ip::protocol::IpProtocol;
use etherparse::NetHeaders;
use serde::Serialize;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_field_names)]
pub struct IpHeader {
    pub ip_header_length: usize,
    pub payload_length: u16,
    pub protocol: IpProtocol,
    pub source_ip: String,
    pub destination_ip: String,
}

impl IpHeader {
    pub fn from_etherparse(net: Option<NetHeaders>) -> Option<Self> {
        match net {
            Some(NetHeaders::Ipv4(h, _)) => {
                let ip_header_length = h.header_len();
                let payload_length = h.payload_len().map_err(|e| eprintln!("{e:?}")).ok()?;
                let protocol = IpProtocol::from_u8(h.protocol.0);

                let source_ip = Ipv4Addr::from(h.source).to_string();
                let destination_ip = Ipv4Addr::from(h.destination).to_string();

                Some(Self {
                    ip_header_length,
                    payload_length,
                    protocol,
                    source_ip,
                    destination_ip,
                })
            }
            Some(NetHeaders::Ipv6(h, _)) => {
                let ip_header_length = h.header_len();
                let payload_length = h.payload_length;
                let protocol = IpProtocol::from_u8(h.next_header.0);

                let source_ip = h.source_addr().to_string();
                let destination_ip = h.destination_addr().to_string();

                Some(Self {
                    ip_header_length,
                    payload_length,
                    protocol,
                    source_ip,
                    destination_ip,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::ip::protocol::IpProtocol;

    #[test]
    fn test_ip_header_from_bytes_success() {
        let raw_data: [u8; 20] = [
            0x45, 0x00, 0x00, 0x3C, // Version, IHL, Total Length
            0x00, 0x00, 0x40, 0x00, // Identification, Flags, Fragment Offset
            0x40, 0x06, 0xB1, 0xE6, // TTL, Protocol (TCP), Header Checksum
            0xC0, 0xA8, 0x01, 0x01, // Source IP: 192.168.1.1
            0xC0, 0xA8, 0x01, 0x02, // Destination IP: 192.168.1.2
        ];

        let etherparse = etherparse::Ipv4Header::from_slice(&raw_data).unwrap().0;
        let header =
            IpHeader::from_etherparse(Some(NetHeaders::Ipv4(etherparse, Default::default())))
                .expect("Failed to parse IP header");

        assert_eq!(header.ip_header_length, 20);
        assert_eq!(header.payload_length, 40);
        assert_eq!(header.protocol, IpProtocol::Tcp);
        assert_eq!(header.source_ip, "192.168.1.1");
        assert_eq!(header.destination_ip, "192.168.1.2");
    }

    #[test]
    fn test_ip_header_to_json() {
        let header = IpHeader {
            ip_header_length: 20,
            payload_length: 40,
            protocol: IpProtocol::Tcp,
            source_ip: "8.8.8.8".to_string(),
            destination_ip: "10.0.0.1".to_string(),
        };

        let json = serde_json::to_string(&header).unwrap();
        let expected = r#"{"ip_header_length":20,"payload_length":40,"protocol":"tcp","source_ip":"8.8.8.8","destination_ip":"10.0.0.1"}"#;
        assert_eq!(json, expected);
    }
}
