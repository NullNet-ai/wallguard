use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Policy {
    #[default]
    Accept,
    Drop,
}

impl FromStr for Policy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "accept" => Ok(Policy::Accept),
            "drop" => Ok(Policy::Drop),
            other => Err(format!("Unknown policy: {}", other)),
        }
    }
}

impl fmt::Display for Policy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Policy::Accept => "accept",
            Policy::Drop => "drop",
        };

        write!(f, "{}", s)
    }
}
