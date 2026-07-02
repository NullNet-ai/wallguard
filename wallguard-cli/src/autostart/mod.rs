#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "freebsd")]
pub use freebsd::{disable_service, enable_service};

#[cfg(target_os = "linux")]
mod linux;
// TODO: enable_service is temporarily unused while its call site in
// main.rs is commented out for testing; remove this allow when re-enabled.
#[cfg(target_os = "linux")]
#[allow(unused_imports)]
pub use linux::{disable_service, enable_service};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{disable_service, enable_service};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{disable_service, enable_service};
