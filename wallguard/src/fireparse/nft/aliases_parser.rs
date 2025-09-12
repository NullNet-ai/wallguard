use crate::fireparse::nft::utils::{nffam2str, nfsettype2str, str2nffam, str2nfsettype};
use nftables::expr::Expression;
use nftables::schema::{NfListObject, NfObject, Nftables, Set, SetType, SetTypeValue};
use nftables::types::NfFamily;
use std::borrow::Cow;
use wallguard_common::protobuf::wallguard_models::Alias;

pub struct NftablesAliasesParser;

impl NftablesAliasesParser {
    pub fn parse(tables: &Nftables) -> Vec<Alias> {
        let mut aliases = vec![];

        for object in tables.objects.iter() {
            if let NfObject::ListObject(NfListObject::Set(set)) = object {
                let SetTypeValue::Single(set_type) = set.set_type else {
                    continue;
                };

                aliases.push(Alias {
                    r#type: nfsettype2str(set_type),
                    name: set.name.to_string(),
                    value: NftablesAliasesParser::expressions_to_csv(&set.elem).unwrap_or_default(),
                    family: nffam2str(set.family),
                    table: set.table.to_string(),
                    ..Default::default()
                });
            }
        }

        aliases
    }

    fn expressions_to_csv<'a>(elem: &Option<Cow<'a, [Expression<'a>]>>) -> Option<String> {
        let Some(expressions) = elem else {
            return None;
        };

        let mut result = Vec::new();

        for e in expressions.iter() {
            match e {
                Expression::String(s) => result.push(s.to_string()),
                Expression::Number(n) => result.push(n.to_string()),
                Expression::Range(range) => match (&range.range[0], &range.range[1]) {
                    (Expression::Number(f), Expression::Number(t)) => {
                        if f <= t {
                            for v in *f..=*t {
                                result.push(v.to_string());
                            }
                        } else {
                            return None;
                        }
                    }
                    (Expression::String(f), Expression::String(t)) => {
                        let f_bytes = f.as_bytes();
                        let t_bytes = t.as_bytes();
                        if f_bytes.len() == 1 && t_bytes.len() == 1 && f_bytes[0] <= t_bytes[0] {
                            for b in f_bytes[0]..=t_bytes[0] {
                                result.push((b as char).to_string());
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                },
                _ => return None,
            }
        }

        Some(result.join(","))
    }

    pub fn convert_alias(alias: Alias) -> Box<Set<'static>> {
        let set_type = str2nfsettype(&alias.r#type).unwrap_or(SetType::Ipv4Addr);

        let mut expressions: Vec<Expression> = Vec::new();

        for item in alias.value.split(',') {
            let item = item.trim();

            if item.is_empty() {
                continue;
            }

            if let Ok(n) = item.parse::<u32>() {
                expressions.push(Expression::Number(n));
            } else {
                expressions.push(Expression::String(Cow::Owned(item.to_string())));
            }
            // @TODO: Ranges support
        }

        Box::new(Set {
            name: alias.name.into(),
            set_type: SetTypeValue::Single(set_type),
            elem: Some(Cow::Owned(expressions)),
            family: str2nffam(&alias.family).unwrap_or(NfFamily::IP),
            table: alias.table.into(),
            ..Default::default()
        })
    }
}
