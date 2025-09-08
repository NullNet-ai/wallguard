use nftables::schema::Rule;

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
}
