use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u16)]
pub enum EtherType {
    #[serde(rename = "ipv4")]
    Ipv4 = 0x0800,
    #[serde(rename = "ipv6")]
    Ipv6 = 0x86DD,
    #[serde(rename = "unknown")]
    Unknown,
}

impl EtherType {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x0800 => Self::Ipv4,
            0x86DD => Self::Ipv6,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ether_type_from_u16() {
        let cases = [
            (0x0800, EtherType::Ipv4),
            (0x86DD, EtherType::Ipv6),
            (0x0806, EtherType::Unknown),
        ];

        for (value, expected) in cases {
            assert_eq!(EtherType::from_u16(value), expected)
        }
    }

    #[test]
    fn test_ether_type_serialize() {
        let cases = [
            (EtherType::Ipv4, r#""ipv4""#),
            (EtherType::Ipv6, r#""ipv6""#),
            (EtherType::Unknown, r#""unknown""#),
        ];

        for (ether_type, expected_str) in cases {
            assert_eq!(serde_json::to_string(&ether_type).unwrap(), expected_str,);
        }
    }
}
