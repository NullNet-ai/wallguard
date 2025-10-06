use crate::fireparse::nft::utils::{NftDirection, nfop2str, str2nfop};
use nftables::expr::{Expression, NamedExpression, Payload, Range, SetItem};
use nftables::schema::Rule;
use nftables::stmt::{Match, Statement};
use wallguard_common::protobuf::wallguard_models::AddrInfo;

pub struct AddrHelper;

impl AddrHelper {
    pub fn extract(rule: &Rule, dir: NftDirection) -> Option<AddrInfo> {
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
                Expression::Named(NamedExpression::Set(set_items)) => {
                    let mut values: Vec<String> = vec![];

                    for item in set_items.iter() {
                        if let SetItem::Element(Expression::String(value)) = item {
                            values.push(value.to_string());
                        };

                        if let SetItem::Element(Expression::Range(range)) = item
                            && let Expression::String(from) = &range.range[0]
                            && let Expression::String(to) = &range.range[1]
                        {
                            values.push(format!("{}-{}", from, to));
                        }
                    }

                    values.join(",")
                }
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

    pub fn build(info: &AddrInfo, dir: NftDirection) -> Option<Statement<'static>> {
        let field_name = match dir {
            NftDirection::Source => "saddr",
            NftDirection::Destination => "daddr",
        };

        let protocol = match info.version {
            4 => "ip",
            6 => "ip6",
            _ => return None,
        };

        let payload_field = Payload::PayloadField(nftables::expr::PayloadField {
            protocol: protocol.into(),
            field: field_name.into(),
        });

        let left = Expression::Named(NamedExpression::Payload(payload_field));

        let right = if info.value.contains(',') {
            let mut items = vec![];
            for part in info.value.split(',') {
                if let Some((from, to)) = part.split_once('-') {
                    items.push(SetItem::Element(Expression::Range(Box::new(Range {
                        range: [
                            Expression::String(from.to_string().into()),
                            Expression::String(to.to_string().into()),
                        ],
                    }))));
                } else {
                    items.push(SetItem::Element(Expression::String(
                        part.to_string().into(),
                    )));
                }
            }
            Expression::Named(NamedExpression::Set(items))
        } else if let Some((from, to)) = info.value.split_once('-') {
            Expression::Range(Box::new(Range {
                range: [
                    Expression::String(from.to_string().into()),
                    Expression::String(to.to_string().into()),
                ],
            }))
        } else {
            Expression::String(info.value.clone().into())
        };

        Some(Statement::Match(Match {
            left,
            right,
            op: str2nfop(&info.operator)?,
        }))
    }
}
