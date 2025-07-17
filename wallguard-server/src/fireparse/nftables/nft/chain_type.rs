use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainType {
    Filter,
    Route,
    Nat,
}

impl FromStr for ChainType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "filter" => Ok(ChainType::Filter),
            "route" => Ok(ChainType::Route),
            "nat" => Ok(ChainType::Nat),
            other => Err(format!("Unknown chain type: {}", other)),
        }
    }
}

impl fmt::Display for ChainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ChainType::Filter => "filter",
            ChainType::Route => "route",
            ChainType::Nat => "nat",
        };

        write!(f, "{}", s)
    }
}
