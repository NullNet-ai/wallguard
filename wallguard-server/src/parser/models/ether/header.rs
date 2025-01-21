use crate::parser::models::ether::r#type::EtherType;
use etherparse::LinkHeader;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EthernetHeader {
    pub source_mac: String,
    pub destination_mac: String,
    pub ether_type: EtherType,
}

impl EthernetHeader {
    pub fn from_etherparse(link: Option<LinkHeader>) -> Option<Self> {
        if let Some(LinkHeader::Ethernet2(link)) = link {
            let format_addr = |v: [u8; 6]| {
                format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    v[0], v[1], v[2], v[3], v[4], v[5]
                )
            };

            let source_mac = format_addr(link.source);
            let destination_mac = format_addr(link.destination);
            let ether_type = EtherType::from_u16(link.ether_type.0);

            Some(Self {
                source_mac,
                destination_mac,
                ether_type,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::ether::r#type::EtherType;

    #[test]
    fn test_ether_header_from_bytes_success() {
        let raw_data: [u8; 14] = [
            0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, // Destination MAC
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, // Source MAC
            0x08, 0x00, // EtherType (IP)
        ];

        let etherparse = LinkHeader::Ethernet2(
            etherparse::Ethernet2Header::from_slice(&raw_data)
                .unwrap()
                .0,
        );
        let header = EthernetHeader::from_etherparse(Some(etherparse))
            .expect("Failed to parse Ethernet header");

        assert_eq!(header.destination_mac, "00:1a:2b:3c:4d:5e");
        assert_eq!(header.source_mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(header.ether_type, EtherType::Ipv4);
    }

    #[test]
    fn test_ether_header_to_json() {
        let header = EthernetHeader {
            source_mac: "00:00:00:00:00:00".to_string(),
            destination_mac: "ff:ff:ff:ff:ff:ff".to_string(),
            ether_type: EtherType::Ipv4,
        };

        let json = serde_json::to_string(&header).unwrap();
        let expected = r#"{"source_mac":"00:00:00:00:00:00","destination_mac":"ff:ff:ff:ff:ff:ff","ether_type":"ipv4"}"#;
        assert_eq!(json, expected);
    }
}
