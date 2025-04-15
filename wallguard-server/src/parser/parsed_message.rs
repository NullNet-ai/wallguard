use crate::parser::models::transport::header::TransportHeader;
use serde::Serialize;
use std::net::IpAddr;

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
    use crate::parser::models::ip::protocol::IpProtocol;
    use std::net::IpAddr;
    use std::str::FromStr;

    const RECORD_1_JSON: &'static str = r#"{"device_id":"machine-id-1234","interface_name":"eth0","timestamp":"2021-08-01T00:00:00Z","total_length":1528,"remote_ip":"8.8.8.8","source_ip":"8.8.8.8","destination_ip":"9.9.9.9","protocol":"tcp","source_port":443,"destination_port":50051}"#;

    const RECORD_2_JSON: &'static str = r#"{"device_id":"machine-id-5678","interface_name":"eth0","timestamp":"2022-09-01T00:00:00Z","total_length":77,"source_ip":"8.8.8.8","destination_ip":"9.9.9.9","protocol":"icmpv4"}"#;

    fn parsed_record_1() -> ParsedRecord {
        ParsedRecord {
            device_id: "machine-id-1234".to_string(),
            interface_name: "eth0".to_string(),
            timestamp: "2021-08-01T00:00:00Z".to_string(),
            total_length: 1528,
            remote_ip: Some(IpAddr::from_str("8.8.8.8").unwrap()),
            ip_header: IpHeader {
                packet_length: 0,
                protocol: IpProtocol::Tcp,
                source_ip: IpAddr::from_str("8.8.8.8").unwrap(),
                destination_ip: IpAddr::from_str("9.9.9.9").unwrap(),
            },
            transport_header: TransportHeader {
                source_port: Some(443),
                destination_port: Some(50051),
            },
        }
    }

    fn parsed_record_2() -> ParsedRecord {
        ParsedRecord {
            device_id: "machine-id-5678".to_string(),
            interface_name: "eth0".to_string(),
            timestamp: "2022-09-01T00:00:00Z".to_string(),
            total_length: 77,
            remote_ip: None,
            ip_header: IpHeader {
                packet_length: 1512,
                protocol: IpProtocol::IcmpV4,
                source_ip: IpAddr::from_str("8.8.8.8").unwrap(),
                destination_ip: IpAddr::from_str("9.9.9.9").unwrap(),
            },
            transport_header: TransportHeader {
                source_port: None,
                destination_port: None,
            },
        }
    }

    #[test]
    fn test_parsed_record_1_to_json() {
        let parsed_record = parsed_record_1();
        let json = serde_json::to_string(&parsed_record).unwrap();
        assert_eq!(json, RECORD_1_JSON);
    }

    #[test]
    fn test_parsed_record_2_to_json() {
        let parsed_record = parsed_record_2();
        let json = serde_json::to_string(&parsed_record).unwrap();
        assert_eq!(json, RECORD_2_JSON);
    }

    #[test]
    fn test_parsed_message_to_json() {
        let records = vec![parsed_record_1(), parsed_record_2()];
        let parsed_message = ParsedMessage { records };
        let json = serde_json::to_string(&parsed_message).unwrap();
        let expected = format!("[{RECORD_1_JSON},{RECORD_2_JSON}]");
        assert_eq!(json, expected.to_string());
    }
}
