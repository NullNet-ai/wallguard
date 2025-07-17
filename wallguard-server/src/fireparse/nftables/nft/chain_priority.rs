use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum Priority {
    NfIpPriConntrackDefrag = -400,
    NfIpPriRaw = -300,
    NfIpPriSelinuxFirst = -225,
    NfIpPriConntrack = -200,
    NfIpPriMangle = -150,
    NfIpPriNatDst = -100,
    NfIpPriFilter = 0,
    NfIpPriSecurity = 50,
    NfIpPriNatSrc = 100,
    NfIpPriSelinuxLast = 225,
    NfIpPriConntrackHelper = 300,
    Other(i32),
}

impl From<i32> for Priority {
    fn from(value: i32) -> Self {
        use Priority::*;
        match value {
            -400 => NfIpPriConntrackDefrag,
            -300 => NfIpPriRaw,
            -225 => NfIpPriSelinuxFirst,
            -200 => NfIpPriConntrack,
            -150 => NfIpPriMangle,
            -100 => NfIpPriNatDst,
            0 => NfIpPriFilter,
            50 => NfIpPriSecurity,
            100 => NfIpPriNatSrc,
            225 => NfIpPriSelinuxLast,
            300 => NfIpPriConntrackHelper,
            value => Other(value),
        }
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "conntrack-defrag" => Ok(Priority::NfIpPriConntrackDefrag),
            "raw" => Ok(Priority::NfIpPriRaw),
            "selinux-first" => Ok(Priority::NfIpPriSelinuxFirst),
            "conntrack" => Ok(Priority::NfIpPriConntrack),
            "mangle" => Ok(Priority::NfIpPriMangle),
            "nat-dst" => Ok(Priority::NfIpPriNatDst),
            "filter" => Ok(Priority::NfIpPriFilter),
            "security" => Ok(Priority::NfIpPriSecurity),
            "nat-src" => Ok(Priority::NfIpPriNatSrc),
            "selinux-last" => Ok(Priority::NfIpPriSelinuxLast),
            "conntrack-helper" => Ok(Priority::NfIpPriConntrackHelper),
            other => match other.parse::<i32>() {
                Ok(value) => Ok(Priority::from(value)),
                Err(_) => Err(format!("Invalid priority value: {}", other)),
            },
        }
    }
}

impl Priority {
    #[allow(unused)]
    pub fn get_value(&self) -> i32 {
        match self {
            Priority::NfIpPriConntrackDefrag => -400,
            Priority::NfIpPriRaw => -300,
            Priority::NfIpPriSelinuxFirst => -225,
            Priority::NfIpPriConntrack => -200,
            Priority::NfIpPriMangle => -150,
            Priority::NfIpPriNatDst => -100,
            Priority::NfIpPriFilter => 0,
            Priority::NfIpPriSecurity => 50,
            Priority::NfIpPriNatSrc => 100,
            Priority::NfIpPriSelinuxLast => 225,
            Priority::NfIpPriConntrackHelper => 300,
            Priority::Other(value) => *value,
        }
    }
}
