/// Returns `true` if the remote desktop capture backend is usable at runtime.
///
/// Compiled without `remote-desktop` feature: always returns `false`.
/// Compiled with the feature: probes the display backend; returns `false`
/// (without panicking) on headless systems or if the backend initialisation
/// fails.  This allows the same binary to run on FreeBSD appliances regardless
/// of whether a display server is present.
pub async fn probe_remote_desktop() -> bool {
    #[cfg(feature = "remote-desktop")]
    {
        // TODO (Phase 8e): replace with real captis::Display::open() probe.
        // Conservative stub: returns false so headless appliances do not
        // accidentally advertise Feature::RemoteDesktop before the capture
        // backend is wired up.
        tracing::info!("remote-desktop: capture probe stub → unavailable (Phase 8e pending)");
        false
    }
    #[cfg(not(feature = "remote-desktop"))]
    false
}
