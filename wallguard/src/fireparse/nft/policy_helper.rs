use nftables::{schema::Rule, stmt::Statement};

pub struct PolicyHelper;

impl PolicyHelper {
    pub fn extract(rule: &Rule) -> Option<String> {
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

    pub fn build(policy: &str) -> Option<Statement<'static>> {
        match policy {
            "accept" => Some(Statement::Accept(None)),
            "drop" => Some(Statement::Drop(None)),
            "reject" => Some(Statement::Reject(None)),
            _ => None,
        }
    }
}
