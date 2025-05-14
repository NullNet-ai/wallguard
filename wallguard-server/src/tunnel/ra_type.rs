use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RAType {
    Shell,
    UI,
    Ssh,
}

impl FromStr for RAType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SHELL" => Ok(RAType::Shell),
            "UI" => Ok(RAType::UI),
            "SSH" => Ok(RAType::Ssh),
            _ => Err(format!("Invalid RAType: {}", s)).handle_err(location!()),
        }
    }
}

impl Display for RAType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RAType::Shell => write!(f, "SHELL"),
            RAType::UI => write!(f, "UI"),
            RAType::Ssh => write!(f, "SSH"),
        }
    }
}
