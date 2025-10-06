use nftables::{
    expr::{Expression, NamedExpression, Payload, Range, SetItem},
    schema::Rule,
    stmt::{Match, Statement},
};
use wallguard_common::protobuf::wallguard_models::PortInfo;

use crate::fireparse::nft::utils::{NftDirection, nfop2str, str2nfop};

pub struct PortHelper;

impl PortHelper {
    pub fn extract(rule: &Rule, dir: NftDirection) -> Option<PortInfo> {
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
                Expression::Named(NamedExpression::Set(set_items)) => {
                    let mut values: Vec<String> = vec![];

                    for item in set_items.iter() {
                        if let SetItem::Element(Expression::Number(value)) = item {
                            values.push(value.to_string());
                        };

                        if let SetItem::Element(Expression::Range(range)) = item
                            && let Expression::Number(from) = &range.range[0]
                            && let Expression::Number(to) = &range.range[1]
                        {
                            values.push(format!("{}-{}", from, to));
                        }
                    }

                    values.join(",")
                }
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

    pub fn build(info: &PortInfo, dir: NftDirection, protocol: &str) -> Option<Statement<'static>> {
        if !matches!(protocol, "tcp" | "udp" | "udplite" | "sctp" | "dccp") {
            return None;
        }

        let field_name = match dir {
            NftDirection::Source => "sport",
            NftDirection::Destination => "dport",
        };

        let payload_field = Payload::PayloadField(nftables::expr::PayloadField {
            protocol: protocol.to_string().into(),
            field: field_name.to_string().into(),
        });

        let left = Expression::Named(NamedExpression::Payload(payload_field));

        let right = if info.value.contains(',') {
            let mut items = vec![];
            for part in info.value.split(',') {
                if let Some((from, to)) = part.split_once('-') {
                    if let (Ok(from_num), Ok(to_num)) = (from.parse::<u32>(), to.parse::<u32>()) {
                        items.push(SetItem::Element(Expression::Range(Box::new(Range {
                            range: [Expression::Number(from_num), Expression::Number(to_num)],
                        }))));
                    }
                } else if let Ok(num) = part.parse::<u32>() {
                    items.push(SetItem::Element(Expression::Number(num)));
                }
            }
            Expression::Named(NamedExpression::Set(items))
        } else if let Some((from, to)) = info.value.split_once('-') {
            if let (Ok(from_num), Ok(to_num)) = (from.parse::<u32>(), to.parse::<u32>()) {
                Expression::Range(Box::new(Range {
                    range: [Expression::Number(from_num), Expression::Number(to_num)],
                }))
            } else {
                return None;
            }
        } else if let Ok(num) = info.value.parse::<u32>() {
            Expression::Number(num)
        } else {
            return None;
        };

        Some(Statement::Match(Match {
            left,
            right,
            op: str2nfop(&info.operator)?,
        }))
    }
}
