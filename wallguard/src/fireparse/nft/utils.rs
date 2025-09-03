use nftables::{expr::SetItem, schema::Rule, stmt::Operator};
use wallguard_common::protobuf::wallguard_models::{AddrInfo, PortInfo};

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

pub enum NftDirection {
    Source,
    Destination,
}

pub fn extract_addr_info(rule: &Rule, dir: NftDirection) -> Option<AddrInfo> {
    use nftables::expr::{Expression, NamedExpression, Payload};
    use nftables::stmt::Statement;

    for statement in rule.expr.iter() {
        let Statement::Match(match_stmt) = statement else {
            continue;
        };

        let Expression::Named(NamedExpression::Payload(Payload::PayloadField(payload_field))) =
            &match_stmt.left
        else {
            continue;
        };

        let field_name = match dir {
            NftDirection::Source => "saddr",
            NftDirection::Destination => "daddr",
        };

        if payload_field.field != field_name {
            continue;
        }

        let version = match payload_field.protocol.as_ref() {
            "ip" => 4,
            "ip6" => 6,
            _ => continue,
        };

        let value = match &match_stmt.right {
            Expression::String(value) => value.to_string(),
            Expression::Named(named_expression) => match named_expression {
                NamedExpression::Set(set_items) => {
                    let mut values: Vec<String> = vec![];

                    for item in set_items.iter() {
                        if let SetItem::Element(Expression::String(value)) = item {
                            values.push(value.to_string());
                        };

                        if let SetItem::Element(Expression::Range(range)) = item {
                            if let Expression::String(from) = &range.range[0] {
                                if let Expression::String(to) = &range.range[1] {
                                    values.push(format!("{}-{}", from, to));
                                }
                            }
                        }
                    }

                    values.join(",")
                }
                _ => continue,
            },
            Expression::Range(range) => {
                let Expression::String(from) = &range.range[0] else {
                    continue;
                };

                let Expression::String(to) = &range.range[1] else {
                    continue;
                };

                format!("{}-{}", from, to)
            }
            _ => continue,
        };

        return Some(AddrInfo {
            version,
            value: value.to_string(),
            operator: nfop2str(match_stmt.op),
        });
    }

    None
}

pub fn extract_port_info(rule: &Rule, dir: NftDirection) -> Option<PortInfo> {
    use nftables::expr::{Expression, NamedExpression, Payload};
    use nftables::stmt::Statement;

    for statement in rule.expr.iter() {
        let Statement::Match(match_stmt) = statement else {
            continue;
        };

        let Expression::Named(NamedExpression::Payload(Payload::PayloadField(payload_field))) =
            &match_stmt.left
        else {
            continue;
        };

        let field_name = match dir {
            NftDirection::Source => "sport",
            NftDirection::Destination => "dport",
        };

        if payload_field.field != field_name {
            continue;
        }

        if !matches!(
            payload_field.protocol.as_ref(),
            "tcp" | "udp" | "udplite" | "sctp" | "dccp"
        ) {
            continue;
        }

        let value = match &match_stmt.right {
            Expression::Number(value) => value.to_string(),
            Expression::Named(named_expression) => match named_expression {
                NamedExpression::Set(set_items) => {
                    let mut values: Vec<String> = vec![];

                    for item in set_items.iter() {
                        if let SetItem::Element(Expression::Number(value)) = item {
                            values.push(value.to_string());
                        };

                        if let SetItem::Element(Expression::Range(range)) = item {
                            if let Expression::Number(from) = &range.range[0] {
                                if let Expression::Number(to) = &range.range[1] {
                                    values.push(format!("{}-{}", from, to));
                                }
                            }
                        }
                    }

                    values.join(",")
                }
                _ => continue,
            },
            Expression::Range(range) => {
                let Expression::Number(from) = &range.range[0] else {
                    continue;
                };

                let Expression::Number(to) = &range.range[1] else {
                    continue;
                };

                format!("{}-{}", from, to)
            }
            _ => continue,
        };

        return Some(PortInfo {
            value,
            operator: nfop2str(match_stmt.op),
        });
    }

    None
}
