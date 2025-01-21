use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UdpHeader {
    pub source_port: u16,
    pub destination_port: u16,
}

impl UdpHeader {
    pub fn from_etherparse(udp: &etherparse::UdpHeader) -> Self {
        Self {
            source_port: udp.source_port,
            destination_port: udp.destination_port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_header_from_bytes_success() {
        let raw_data: [u8; 8] = [
            0x1F, 0x90, 0x00, 0x35, // Source Port: 8080, Destination Port: 53
            0x00, 0x1C, // Length: 28
            0x1C, 0x46, // Checksum: 7238
        ];

        let etherparse = etherparse::UdpHeader::from_slice(&raw_data).unwrap().0;
        let header = UdpHeader::from_etherparse(&etherparse);

        assert_eq!(header.source_port, 8080);
        assert_eq!(header.destination_port, 53);
    }
}
