use nftables::schema::Rule;

pub struct L4ProtocolHelper;

impl L4ProtocolHelper {
    pub fn extract(rule: &Rule) -> Option<String> {
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
}
