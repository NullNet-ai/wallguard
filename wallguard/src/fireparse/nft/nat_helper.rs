use nftables::{
    expr::Expression,
    schema::Rule,
    stmt::{NAT, Statement},
};

pub struct NatHelper;

impl NatHelper {
    pub fn extract(rule: &Rule) -> Option<(Option<String>, Option<u32>)> {
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

    pub fn build(addr: Option<String>, port: Option<u32>) -> Option<Statement<'static>> {
        if addr.is_none() && port.is_none() {
            return None;
        }

        let nat = NAT {
            addr: addr.map(|a| Expression::String(a.into())),
            port: port.map(Expression::Number),
            family: Default::default(),
            flags: Default::default(),
        };

        Some(Statement::DNAT(Some(nat)))
    }
}
