use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_field_names)]
pub struct TcpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub tcp_header_length: usize,
    pub tcp_sequence_number: u32,
    pub tcp_acknowledgment_number: u32,
    pub tcp_data_offset: u8,
    pub tcp_flags: u8,
    pub tcp_window_size: u16,
    pub tcp_urgent_pointer: u16,
}

impl TcpHeader {
    pub fn from_etherparse(tcp: &etherparse::TcpHeader) -> Self {
        let tcp_flags = u8::from(tcp.cwr)
            | (u8::from(tcp.ece) << 1)
            | (u8::from(tcp.urg) << 2)
            | (u8::from(tcp.ack) << 3)
            | (u8::from(tcp.psh) << 4)
            | (u8::from(tcp.rst) << 5)
            | (u8::from(tcp.syn) << 6)
            | (u8::from(tcp.fin) << 7);

        Self {
            source_port: tcp.source_port,
            destination_port: tcp.destination_port,
            tcp_header_length: tcp.header_len(),
            tcp_sequence_number: tcp.sequence_number,
            tcp_acknowledgment_number: tcp.acknowledgment_number,
            tcp_data_offset: tcp.data_offset(),
            tcp_flags,
            tcp_window_size: tcp.window_size,
            tcp_urgent_pointer: tcp.urgent_pointer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_header_from_bytes_success() {
        let raw_data: [u8; 20] = [
            0x1F, 0x90, 0x00, 0x50, // Source Port: 8080, Destination Port: 80
            0x00, 0x00, 0x00, 0x01, // Sequence Number: 1
            0x00, 0x00, 0x00, 0x02, // Acknowledgment Number: 2
            0x50, 0x18, // Data Offset: 5, Flags: ACK
            0x72, 0x10, // Window Size: 29200
            0x1C, 0x46, // Checksum: 7238
            0x00, 0x00, // Urgent Pointer: 0
        ];

        let etherparse = etherparse::TcpHeader::from_slice(&raw_data).unwrap().0;
        let header = TcpHeader::from_etherparse(&etherparse);

        assert_eq!(header.source_port, 8080);
        assert_eq!(header.destination_port, 80);
        assert_eq!(header.tcp_sequence_number, 1);
        assert_eq!(header.tcp_acknowledgment_number, 2);
        assert_eq!(header.tcp_data_offset, 5);
        assert_eq!(header.tcp_flags, 0x18);
        assert_eq!(header.tcp_window_size, 29200);
        assert_eq!(header.tcp_urgent_pointer, 0);
    }
}
