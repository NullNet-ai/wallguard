#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "freebsd")]
pub use freebsd::{disable_service, enable_service};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{disable_service, enable_service};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{disable_service, enable_service};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{disable_service, enable_service};
