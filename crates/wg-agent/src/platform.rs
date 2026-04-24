/// Compile-time target OS identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOs {
    Linux,
    FreeBsd,
    Windows,
    Other,
}

pub const TARGET_OS: TargetOs = {
    #[cfg(target_os = "linux")]
    { TargetOs::Linux }
    #[cfg(target_os = "freebsd")]
    { TargetOs::FreeBsd }
    #[cfg(target_os = "windows")]
    { TargetOs::Windows }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "windows",
    )))]
    { TargetOs::Other }
};
