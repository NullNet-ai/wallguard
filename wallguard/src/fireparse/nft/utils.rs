use nftables::stmt::Operator;

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

pub enum NftDirection {
    Source,
    Destination,
}
