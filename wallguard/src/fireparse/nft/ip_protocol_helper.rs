use nftables::{schema::Rule, stmt::Statement};

pub struct IpProtocolHelper;

impl IpProtocolHelper {
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

            match payload_field.protocol.as_ref() {
                "ip" => return Some("ip".to_string()),
                "ip6" => return Some("ip6".to_string()),
                _ => continue,
            }
        }

        None
    }

    pub fn build(_protocol: &str) -> Option<Statement<'static>> {
        // match protocol {
        //     "ip" | "ip6" => {
        //         let payload_field = PayloadField {
        //             field: "protocol".to_string().into(),
        //             protocol: protocol.to_string().into(),
        //         };

        //         let left = Expression::Named(NamedExpression::Payload(Payload::PayloadField(
        //             payload_field,
        //         )));

        //         let right = Expression::String(protocol.to_string().into());

        //         let match_stmt = Match {
        //             left,
        //             right,
        //             op: Operator::EQ,
        //         };

        //         Some(Statement::Match(match_stmt))
        //     }
        //     _ => None,
        // }

        None
    }
}
