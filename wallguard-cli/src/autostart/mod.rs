#[cfg(target_os = "freebsd")]
mod freebsd;
// TODO: enable_service is temporarily unused while its call site in
// main.rs is commented out for testing; remove this allow when re-enabled.
#[cfg(target_os = "freebsd")]
#[allow(unused_imports)]
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
// TODO: enable_service is temporarily unused while its call site in
// main.rs is commented out for testing; remove this allow when re-enabled.
#[cfg(target_os = "macos")]
#[allow(unused_imports)]
pub use macos::{disable_service, enable_service};

#[cfg(windows)]
mod windows;
// TODO: enable_service is temporarily unused while its call site in
// main.rs is commented out for testing; remove this allow when re-enabled.
#[cfg(windows)]
#[allow(unused_imports)]
pub use windows::{disable_service, enable_service};
