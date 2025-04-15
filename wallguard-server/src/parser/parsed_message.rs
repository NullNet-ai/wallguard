use crate::parser::connections_map::{ConnectionKey, ConnectionValue};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct ParsedMessage {
    pub records: Vec<ParsedRecord>,
}

#[derive(Debug, Serialize)]
pub struct ParsedRecord {
    #[serde(flatten)]
    pub connection_key: ConnectionKey,
    #[serde(flatten)]
    pub connection_value: ConnectionValue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ip_header::IpHeader;
    use crate::parser::transport_header::{Protocol, TransportHeader};
    use std::net::IpAddr;
    use std::str::FromStr;

    const RECORD_1_JSON: &'static str = r#"{"device_id":"machine-id-1234","interface_name":"eth0","source_ip":"8.8.8.8","destination_ip":"9.9.9.9","source_port":443,"destination_port":50051,"protocol":"tcp","timestamp":"2021-08-01T00:00:00Z","packets":11,"bytes":1528,"remote_ip":"8.8.8.8"}"#;

    const RECORD_2_JSON: &'static str = r#"{"device_id":"machine-id-5678","interface_name":"eth0","source_ip":"8.8.8.8","destination_ip":"9.9.9.9","protocol":"icmpv4","timestamp":"2022-09-01T00:00:00Z","packets":1,"bytes":77}"#;

    fn parsed_record_1() -> ParsedRecord {
        let key = ConnectionKey {
            device_id: "machine-id-1234".to_string(),
            interface_name: "eth0".to_string(),
            ip_header: IpHeader {
                packet_length: 0,
                source_ip: IpAddr::from_str("8.8.8.8").unwrap(),
                destination_ip: IpAddr::from_str("9.9.9.9").unwrap(),
            },
            transport_header: TransportHeader {
                source_port: Some(443),
                destination_port: Some(50051),
                protocol: Protocol::Tcp,
            },
        };
        let value = ConnectionValue {
            timestamp: "2021-08-01T00:00:00Z".to_string(),
            packets: 11,
            bytes: 1528,
            remote_ip: Some(IpAddr::from_str("8.8.8.8").unwrap()),
        };

        ParsedRecord {
            connection_key: key,
            connection_value: value,
        }
    }

    fn parsed_record_2() -> ParsedRecord {
        let key = ConnectionKey {
            device_id: "machine-id-5678".to_string(),
            interface_name: "eth0".to_string(),
            ip_header: IpHeader {
                packet_length: 1512,
                source_ip: IpAddr::from_str("8.8.8.8").unwrap(),
                destination_ip: IpAddr::from_str("9.9.9.9").unwrap(),
            },
            transport_header: TransportHeader {
                source_port: None,
                destination_port: None,
                protocol: Protocol::IcmpV4,
            },
        };
        let value = ConnectionValue {
            timestamp: "2022-09-01T00:00:00Z".to_string(),
            packets: 1,
            bytes: 77,
            remote_ip: None,
        };

        ParsedRecord {
            connection_key: key,
            connection_value: value,
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
