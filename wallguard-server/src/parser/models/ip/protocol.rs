use crate::parser::models::ip::protocol::IpProtocol::Unknown;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash)]
#[repr(u8)]
pub enum IpProtocol {
    #[serde(rename = "icmpv4")]
    IcmpV4 = 1,
    #[serde(rename = "tcp")]
    Tcp = 6,
    #[serde(rename = "udp")]
    Udp = 17,
    #[serde(rename = "icmpv6")]
    IcmpV6 = 58,
    #[serde(rename = "unknown")]
    Unknown,
}

impl IpProtocol {
    pub fn from_u8(value: u8) -> Self {
        // https://en.wikipedia.org/wiki/List_of_IP_protocol_numbers
        match value {
            1 => Self::IcmpV4,
            6 => Self::Tcp,
            17 => Self::Udp,
            58 => Self::IcmpV6,
            _ => Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_protocol_from_u8() {
        assert_eq!(IpProtocol::from_u8(6), IpProtocol::Tcp);
        assert_eq!(IpProtocol::from_u8(17), IpProtocol::Udp);
        assert_eq!(IpProtocol::from_u8(1), IpProtocol::IcmpV4);
        assert_eq!(IpProtocol::from_u8(58), IpProtocol::IcmpV6);
        assert_eq!(IpProtocol::from_u8(100), IpProtocol::Unknown);
    }

    #[test]
    fn test_ip_protocol_serialize() {
        assert_eq!(serde_json::to_string(&IpProtocol::Tcp).unwrap(), r#""tcp""#);
        assert_eq!(serde_json::to_string(&IpProtocol::Udp).unwrap(), r#""udp""#);
        assert_eq!(
            serde_json::to_string(&IpProtocol::IcmpV4).unwrap(),
            r#""icmpv4""#
        );
        assert_eq!(
            serde_json::to_string(&IpProtocol::IcmpV6).unwrap(),
            r#""icmpv6""#
        );
        assert_eq!(
            serde_json::to_string(&IpProtocol::Unknown).unwrap(),
            r#""unknown""#
        );
    }
}
