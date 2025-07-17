use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hook {
    Ingress,
    Prerouting,
    Forward,
    Input,
    Output,
    Postrouting,
    Egress,
}

impl fmt::Display for Hook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Hook::Ingress => "ingress",
            Hook::Prerouting => "prerouting",
            Hook::Forward => "forward",
            Hook::Input => "input",
            Hook::Output => "output",
            Hook::Postrouting => "postrouting",
            Hook::Egress => "egress",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for Hook {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ingress" => Ok(Hook::Ingress),
            "prerouting" => Ok(Hook::Prerouting),
            "forward" => Ok(Hook::Forward),
            "input" => Ok(Hook::Input),
            "output" => Ok(Hook::Output),
            "postrouting" => Ok(Hook::Postrouting),
            "egress" => Ok(Hook::Egress),
            other => Err(format!("Unknown hook: {}", other)),
        }
    }
}