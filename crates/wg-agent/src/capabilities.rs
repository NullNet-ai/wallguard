/// Returns `true` if the remote desktop capture backend is usable at runtime.
///
/// Opens the platform capture backend; returns `false` (without panicking) on
/// headless systems or if backend initialisation fails.  This lets the same
/// binary run on firewall appliances regardless of whether a display server is
/// present.
pub async fn probe_remote_desktop() -> bool {
    match crate::capture::open_capture_backend() {
        Ok(_) => {
            tracing::info!("remote-desktop: capture backend available");
            true
        }
        Err(e) => {
            tracing::info!("remote-desktop: capture backend unavailable — {e:#}");
            false
        }
    }
}
