use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    #[default]
    Ip,
    Arp,
    Ip6,
    Bridge,
    Inet,
    Netdev,
}

impl FromStr for Family {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ip" => Ok(Family::Ip),
            "arp" => Ok(Family::Arp),
            "ip6" => Ok(Family::Ip6),
            "bridge" => Ok(Family::Bridge),
            "inet" => Ok(Family::Inet),
            "netdev" => Ok(Family::Netdev),
            other => Err(format!("Unknown family: {}", other)),
        }
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Family::Ip => "ip",
            Family::Arp => "arp",
            Family::Ip6 => "ip6",
            Family::Bridge => "bridge",
            Family::Inet => "inet",
            Family::Netdev => "netdev",
        };
        write!(f, "{}", s)
    }
}
