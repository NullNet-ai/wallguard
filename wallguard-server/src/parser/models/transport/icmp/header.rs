use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct IcmpHeader {
    pub icmp_type: u8,
    pub icmp_code: u8,
}

impl IcmpHeader {
    pub fn from_etherparse_v4(header: &etherparse::Icmpv4Header) -> Self {
        let bytes = header.to_bytes();
        let icmp_type = bytes[0];
        let icmp_code = bytes[1];
        IcmpHeader {
            icmp_type,
            icmp_code,
        }
    }

    pub fn from_etherparse_v6(header: &etherparse::Icmpv6Header) -> Self {
        IcmpHeader {
            icmp_type: header.icmp_type.type_u8(),
            icmp_code: header.icmp_type.code_u8(),
        }
    }
}
