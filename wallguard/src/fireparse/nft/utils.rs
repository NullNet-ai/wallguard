use nftables::{
    expr::{Expression, SetItem},
    schema::Rule,
    stmt::Operator,
};
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

pub fn extract_policy(rule: &Rule) -> Option<String> {
    use nftables::stmt::Statement;

    for statement in rule.expr.iter() {
        match statement {
            Statement::Accept(_) => return Some(String::from("accept")),
            Statement::Drop(_) => return Some(String::from("drop")),
            Statement::Reject(_) => return Some(String::from("reject")),
            _ => continue,
        }
    }

    None
}

pub fn extract_nat(rule: &Rule) -> Option<(Option<String>, Option<u32>)> {
    for statement in rule.expr.iter() {
        match statement {
            nftables::stmt::Statement::DNAT(nat) => {
                if let Some(nat_value) = nat {
                    let addr = nat_value.addr.clone().and_then(|value| {
                        if let Expression::String(addr) = value {
                            Some(addr.to_string())
                        } else {
                            None
                        }
                    });

                    let port = nat_value.port.clone().and_then(|value| {
                        if let Expression::Number(port) = value {
                            Some(port)
                        } else {
                            None
                        }
                    });

                    return Some((addr, port));
                }
            }
            _ => continue,
        };
    }

    None
}

pub fn extract_ip_protocol(rule: &Rule) -> Option<String> {
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

        match payload_field.protocol.as_ref() {
            "ip" => return Some("ip".to_string()),
            "ip6" => return Some("ip6".to_string()),
            _ => continue,
        }
    }

    None
}

pub fn extract_l4_protocol(rule: &Rule) -> Option<String> {
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

        if payload_field.field == "protocol"
            && (payload_field.protocol == "ip" || payload_field.protocol == "ip6")
        {
            if let Expression::String(proto) = &match_stmt.right {
                return Some(proto.to_string());
            }

            if let Expression::Number(value) = &match_stmt.right {
                let proto = match *value {
                    1 => "icmp",
                    6 => "tcp",
                    17 => "udp",
                    33 => "dccp",
                    132 => "sctp",
                    136 => "udplite",
                    _ => return Some(format!("proto-{}", value)),
                };
                return Some(proto.to_string());
            }
        }

        if matches!(payload_field.field.as_ref(), "sport" | "dport") {
            return Some(payload_field.protocol.to_string());
        }
    }

    None
}

pub fn extract_interface(rule: &Rule) -> Option<String> {
    use nftables::expr::{Expression, NamedExpression};
    use nftables::stmt::Statement;

    for statement in rule.expr.iter() {
        let Statement::Match(match_stmt) = statement else {
            continue;
        };

        let Expression::Named(NamedExpression::Meta(_)) = &match_stmt.left else {
            continue;
        };

        if let Expression::String(iface_name) = &match_stmt.right {
            return Some(iface_name.to_string());
        }
    }

    None
}
