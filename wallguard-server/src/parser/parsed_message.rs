use crate::parser::models::transport::header::TransportHeader;
use serde::Serialize;
use std::net::IpAddr;

use super::models::ether::header::EthernetHeader;
use super::models::ip::header::IpHeader;

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct ParsedMessage {
    pub records: Vec<ParsedRecord>,
}

#[derive(Debug, Serialize)]
pub struct ParsedRecord {
    pub device_id: String,
    pub interface_name: String,
    pub timestamp: String,
    pub total_length: u16,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub ethernet_header: Option<EthernetHeader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_ip: Option<IpAddr>,
    #[serde(flatten)]
    pub ip_header: IpHeader,
    #[serde(flatten)]
    pub transport_header: TransportHeader,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::models::ether::r#type::EtherType;
    use crate::parser::models::ip::protocol::IpProtocol;
    use crate::parser::models::transport::tcp::header::TcpHeader;
    use crate::parser::models::transport::udp::header::UdpHeader;
    use std::net::IpAddr;
    use std::str::FromStr;

    const ETHERNET_IPV4_TCP_JSON: &'static str = r#"{"device_id":"machine-id-1234","interface_name":"eth0","timestamp":"2021-08-01T00:00:00Z","total_length":1528,"source_mac":"00:00:00:00:00:00","destination_mac":"ff:ff:ff:ff:ff:ff","ether_type":"ipv4","remote_ip":"8.8.8.8","protocol":"tcp","source_ip":"8.8.8.8","destination_ip":"9.9.9.9","source_port":443,"destination_port":50051,"tcp_header_length":20,"tcp_sequence_number":177,"tcp_acknowledgment_number":911,"tcp_data_offset":64,"tcp_flags":56,"tcp_window_size":256,"tcp_urgent_pointer":2}"#;

    const IPV4_UDP_JSON: &'static str = r#"{"device_id":"machine-id-5678","interface_name":"eth0","timestamp":"2022-09-01T00:00:00Z","total_length":77,"protocol":"udp","source_ip":"8.8.8.8","destination_ip":"9.9.9.9","source_port":80,"destination_port":50052}"#;

    fn parsed_record_ethernet_ipv4_tcp() -> ParsedRecord {
        ParsedRecord {
            device_id: "machine-id-1234".to_string(),
            interface_name: "eth0".to_string(),
            timestamp: "2021-08-01T00:00:00Z".to_string(),
            total_length: 1528,
            ethernet_header: Some(EthernetHeader {
                source_mac: "00:00:00:00:00:00".to_string(),
                destination_mac: "ff:ff:ff:ff:ff:ff".to_string(),
                ether_type: EtherType::Ipv4,
            }),
            remote_ip: Some(IpAddr::from_str("8.8.8.8").unwrap()),
            ip_header: IpHeader {
                packet_length: 0,
                protocol: IpProtocol::Tcp,
                source_ip: IpAddr::from_str("8.8.8.8").unwrap(),
                destination_ip: IpAddr::from_str("9.9.9.9").unwrap(),
            },
            transport_header: TransportHeader::Tcp(TcpHeader {
                source_port: 443,
                destination_port: 50051,
                tcp_header_length: 20,
                tcp_sequence_number: 177,
                tcp_acknowledgment_number: 911,
                tcp_data_offset: 64,
                tcp_flags: 56,
                tcp_window_size: 256,
                tcp_urgent_pointer: 2,
            }),
        }
    }

    fn parsed_record_ipv4_udp() -> ParsedRecord {
        ParsedRecord {
            device_id: "machine-id-5678".to_string(),
            interface_name: "eth0".to_string(),
            timestamp: "2022-09-01T00:00:00Z".to_string(),
            total_length: 77,
            ethernet_header: None,
            remote_ip: None,
            ip_header: IpHeader {
                packet_length: 1512,
                protocol: IpProtocol::Udp,
                source_ip: IpAddr::from_str("8.8.8.8").unwrap(),
                destination_ip: IpAddr::from_str("9.9.9.9").unwrap(),
            },
            transport_header: TransportHeader::Udp(UdpHeader {
                source_port: 80,
                destination_port: 50052,
            }),
        }
    }

    #[test]
    fn test_parsed_record_ethernet_ipv4_tcp_to_json() {
        let parsed_record = parsed_record_ethernet_ipv4_tcp();
        let json = serde_json::to_string(&parsed_record).unwrap();
        assert_eq!(json, ETHERNET_IPV4_TCP_JSON);
    }

    #[test]
    fn test_parsed_record_ipv4_udp_to_json() {
        let parsed_record = parsed_record_ipv4_udp();
        let json = serde_json::to_string(&parsed_record).unwrap();
        assert_eq!(json, IPV4_UDP_JSON);
    }

    #[test]
    fn test_parsed_message_to_json() {
        let records = vec![parsed_record_ethernet_ipv4_tcp(), parsed_record_ipv4_udp()];
        let parsed_message = ParsedMessage { records };
        let json = serde_json::to_string(&parsed_message).unwrap();
        let expected = format!("[{ETHERNET_IPV4_TCP_JSON},{IPV4_UDP_JSON}]");
        assert_eq!(json, expected.to_string());
    }
}
