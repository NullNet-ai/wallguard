use nftables::{schema::SetType, stmt::Operator, types::NfFamily};

pub fn nfop2str(operator: Operator) -> String {
    let value = match operator {
        Operator::AND => "and",
        Operator::OR => "or",
        Operator::XOR => "xor",
        Operator::LSHIFT => "lshift",
        Operator::RSHIFT => "rshift",
        Operator::EQ => "eq",
        Operator::NEQ => "neq",
        Operator::LT => "lt",
        Operator::GT => "gt",
        Operator::LEQ => "leq",
        Operator::GEQ => "geq",
        Operator::IN => "in",
    };

    value.to_string()
}

pub fn str2nfop(value: &str) -> Option<Operator> {
    match value {
        "and" => Some(Operator::AND),
        "or" => Some(Operator::OR),
        "xor" => Some(Operator::XOR),
        "lshift" => Some(Operator::LSHIFT),
        "rshift" => Some(Operator::RSHIFT),
        "eq" => Some(Operator::EQ),
        "neq" => Some(Operator::NEQ),
        "lt" => Some(Operator::LT),
        "gt" => Some(Operator::GT),
        "leq" => Some(Operator::LEQ),
        "geq" => Some(Operator::GEQ),
        "in" => Some(Operator::IN),
        _ => None,
    }
}

pub fn nffam2str(family: NfFamily) -> String {
    let str = match family {
        NfFamily::IP => "ip",
        NfFamily::IP6 => "ip6",
        NfFamily::INet => "inet",
        NfFamily::ARP => "arp",
        NfFamily::Bridge => "bridge",
        NfFamily::NetDev => "netdev",
    };

    str.to_string()
}

pub fn str2nffam(value: &str) -> Option<NfFamily> {
    match value.to_lowercase().as_str() {
        "ip" => Some(NfFamily::IP),
        "ip6" => Some(NfFamily::IP6),
        "inet" => Some(NfFamily::INet),
        "arp" => Some(NfFamily::ARP),
        "bridge" => Some(NfFamily::Bridge),
        "netdev" => Some(NfFamily::NetDev),
        _ => None,
    }
}

pub fn nfsettype2str(value: SetType) -> String {
    let retval = match value {
        SetType::Ipv4Addr => "ip",
        SetType::Ipv6Addr => "ip6",
        SetType::EtherAddr => "ether",
        SetType::InetProto => "proto",
        SetType::InetService => "service",
        SetType::Mark => "mark",
        SetType::Ifname => "interface",
    };

    retval.into()
}

pub fn str2nfsettype(value: &str) -> Option<SetType> {
    match value {
        "ip" => Some(SetType::Ipv4Addr),
        "ip6" => Some(SetType::Ipv6Addr),
        "ether" => Some(SetType::EtherAddr),
        "proto" => Some(SetType::InetProto),
        "service" => Some(SetType::InetService),
        "mark" => Some(SetType::Mark),
        "interface" => Some(SetType::Ifname),
        _ => None,
    }
}

pub enum NftDirection {
    Source,
    Destination,
}
