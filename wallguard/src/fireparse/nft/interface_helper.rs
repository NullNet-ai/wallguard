use nftables::{
    expr::{Expression, Meta, MetaKey, NamedExpression},
    schema::Rule,
    stmt::{Match, Operator, Statement},
};

pub struct InterfaceHelper;

impl InterfaceHelper {
    pub fn extract(rule: &Rule) -> Option<String> {
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

    pub fn build(interface_name: &str) -> Statement<'static> {
        Statement::Match(Match {
            left: Expression::Named(NamedExpression::Meta(Meta { key: MetaKey::Iif })),
            op: Operator::EQ,
            right: Expression::String(interface_name.to_string().into()),
        })
    }
}
