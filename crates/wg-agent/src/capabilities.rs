/// Returns `true` if the remote desktop capture backend is usable at runtime.
///
/// Compiled without `remote-desktop` feature: always returns `false`.
/// Compiled with the feature: opens the platform capture backend; returns
/// `false` (without panicking) on headless systems or if the backend
/// initialisation fails.  This lets the same binary run on FreeBSD appliances
/// regardless of whether a display server is present.
pub async fn probe_remote_desktop() -> bool {
    #[cfg(feature = "remote-desktop")]
    {
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
    #[cfg(not(feature = "remote-desktop"))]
    false
}
