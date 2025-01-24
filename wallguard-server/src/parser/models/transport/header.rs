use crate::parser::models::transport::icmp::header::IcmpHeader;
use crate::parser::models::transport::tcp::header::TcpHeader;
use crate::parser::models::transport::udp::header::UdpHeader;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum TransportHeader {
    #[serde(untagged)]
    Udp(UdpHeader),
    #[serde(untagged)]
    Tcp(TcpHeader),
    #[serde(untagged)]
    Icmp(IcmpHeader),
}

impl TransportHeader {
    pub fn from_etherparse(transport: Option<etherparse::TransportHeader>) -> Option<Self> {
        match transport {
            Some(etherparse::TransportHeader::Tcp(h)) => {
                let tcp_header = TcpHeader::from_etherparse(&h);
                Some(TransportHeader::Tcp(tcp_header))
            }
            Some(etherparse::TransportHeader::Udp(h)) => {
                let udp_header = UdpHeader::from_etherparse(&h);
                Some(TransportHeader::Udp(udp_header))
            }
            Some(etherparse::TransportHeader::Icmpv4(h)) => {
                let icmp_header = IcmpHeader::from_etherparse_v4(&h);
                Some(TransportHeader::Icmp(icmp_header))
            }
            Some(etherparse::TransportHeader::Icmpv6(h)) => {
                let icmp_header = IcmpHeader::from_etherparse_v6(&h);
                Some(TransportHeader::Icmp(icmp_header))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::transport::tcp::header::TcpHeader;

    #[test]
    fn test_transport_header_tcp_to_json() {
        let tcp_header = TcpHeader {
            source_port: 443,
            destination_port: 50051,
            tcp_header_length: 20,
            tcp_sequence_number: 123456789,
            tcp_acknowledgment_number: 987654321,
            tcp_data_offset: 5,
            tcp_flags: 0b00000010,
            tcp_window_size: 65535,
            tcp_urgent_pointer: 0x5678,
        };

        let transport_header = TransportHeader::Tcp(tcp_header);
        let json = serde_json::to_string(&transport_header).unwrap();

        assert_eq!(
            json,
            r#"{"source_port":443,"destination_port":50051,"tcp_header_length":20,"tcp_sequence_number":123456789,"tcp_acknowledgment_number":987654321,"tcp_data_offset":5,"tcp_flags":2,"tcp_window_size":65535,"tcp_urgent_pointer":22136}"#
        );
    }

    #[test]
    fn test_transport_header_udp_to_json() {
        let udp_header = UdpHeader {
            source_port: 443,
            destination_port: 50051,
        };

        let transport_header = TransportHeader::Udp(udp_header);
        let json = serde_json::to_string(&transport_header).unwrap();

        assert_eq!(json, r#"{"source_port":443,"destination_port":50051}"#);
    }

    #[test]
    fn test_transport_header_icmp_to_json() {
        let icmp_header = IcmpHeader {
            icmp_type: 8,
            icmp_code: 0,
        };

        let transport_header = TransportHeader::Icmp(icmp_header);
        let json = serde_json::to_string(&transport_header).unwrap();

        assert_eq!(json, r#"{"icmp_type":8,"icmp_code":0}"#);
    }
}
