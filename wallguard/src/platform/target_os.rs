#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOs {
    Linux,
    MacOS,
    Windows,
    FreeBSD,
    OpenBSD,
    NetBSD,
    Unknown,
}

impl Default for TargetOs {
    #[allow(unreachable_code)]
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        {
            return TargetOs::Linux;
        }

        #[cfg(target_os = "windows")]
        {
            return TargetOs::Windows;
        }

        #[cfg(target_os = "macos")]
        {
            return TargetOs::MacOS;
        }

        #[cfg(target_os = "freebsd")]
        {
            return TargetOs::FreeBSD;
        }

        #[cfg(target_os = "openbsd")]
        {
            return TargetOs::OpenBSD;
        }

        #[cfg(target_os = "netbsd")]
        {
            return TargetOs::NetBSD;
        }

        TargetOs::Unknown
    }
}

impl ToString for TargetOs {
    fn to_string(&self) -> String {
        let value = match self {
            TargetOs::Linux => "linux",
            TargetOs::MacOS => "macos",
            TargetOs::Windows => "windows",
            TargetOs::FreeBSD => "freebsd",
            TargetOs::OpenBSD => "openbsd",
            TargetOs::NetBSD => "netbsd",
            TargetOs::Unknown => "unknown",
        };

        String::from(value)
    }
}

impl TargetOs {
    pub fn new() -> Self {
        Self::default()
    }
}