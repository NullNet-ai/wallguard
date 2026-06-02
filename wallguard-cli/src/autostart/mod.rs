// TODO: autostart / service registration is not yet implemented.
// The platform-specific implementations (linux.rs, macos.rs, freebsd.rs, windows.rs)
// are kept for reference but not compiled.

use std::io;

pub async fn enable_service(_program: &str, _args: &[&str]) -> io::Result<()> {
    Err(io::Error::other("autostart not yet implemented"))
}

pub async fn disable_service(_program: &str) -> io::Result<()> {
    Err(io::Error::other("autostart not yet implemented"))
}
