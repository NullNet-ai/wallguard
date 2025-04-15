use crate::parser::models::ip::protocol::IpProtocol;
use etherparse::NetHeaders;
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_field_names)]
pub struct IpHeader {
    #[serde(skip)]
    pub packet_length: u16,
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub protocol: IpProtocol,
}

impl IpHeader {
    pub fn from_etherparse(net: Option<NetHeaders>) -> Option<Self> {
        match net {
            Some(NetHeaders::Ipv4(h, _)) => {
                let packet_length = h.total_len;
                let protocol = IpProtocol::from_u8(h.protocol.0);

                let source_ip = IpAddr::V4(Ipv4Addr::from(h.source));
                let destination_ip = IpAddr::V4(Ipv4Addr::from(h.destination));

                Some(Self {
                    packet_length,
                    protocol,
                    source_ip,
                    destination_ip,
                })
            }
            Some(NetHeaders::Ipv6(h, _)) => {
                let packet_length = 40 + h.payload_length;
                let protocol = IpProtocol::from_u8(h.next_header.0);

                let source_ip = IpAddr::V6(h.source_addr());
                let destination_ip = IpAddr::V6(h.destination_addr());

                Some(Self {
                    packet_length,
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
    use std::str::FromStr;

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

        assert_eq!(header.packet_length, 60);
        assert_eq!(header.protocol, IpProtocol::Tcp);
        assert_eq!(header.source_ip, IpAddr::from_str("192.168.1.1").unwrap());
        assert_eq!(
            header.destination_ip,
            IpAddr::from_str("192.168.1.2").unwrap()
        );
    }

    #[test]
    fn test_ip_header_to_json() {
        let header = IpHeader {
            packet_length: 60,
            protocol: IpProtocol::Tcp,
            source_ip: IpAddr::from_str("8.8.8.8").unwrap(),
            destination_ip: IpAddr::from_str("10.0.0.1").unwrap(),
        };

        let json = serde_json::to_string(&header).unwrap();
        let expected = r#"{"source_ip":"8.8.8.8","destination_ip":"10.0.0.1","protocol":"tcp"}"#;
        assert_eq!(json, expected);
    }
}
