use crate::netinfo::sock::SocketInfo;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::netinfo::service::ServiceInfo;

/// Reports the synthetic Remote Desktop service entry only when the current
/// process can actually connect to the display server.
///
/// The check is intentionally performed on every service-discovery cycle
/// (every 5 minutes) so that:
/// * A logged-out machine reports no RD service (Enigo fails → return empty).
/// * After a user logs in the next cycle picks it up automatically.
/// * The agent never needs to be restarted to regain RD availability.
pub fn filter(_: &mut Vec<SocketInfo>) -> Vec<ServiceInfo> {
    if !is_rd_available() {
        return vec![];
    }

    vec![ServiceInfo {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        protocol: super::Protocol::RemoteDesktop,
        program: String::from("/wallguard-rd"),
    }]
}

/// Returns `true` when a live display session is reachable by the agent
/// process right now.
///
/// The check uses two strategies depending on which display protocol is
/// available:
///
/// **X11 (or XWayland)** — `x11rb::connect()` is attempted first.  This is
/// exactly what `X11Capturer::new()` does, so success here means screen
/// capture will work.  We additionally probe `Enigo::new()` to confirm that
/// input injection is also possible.
///
/// **Pure Wayland (no X11 socket reachable)** — Enigo only speaks X11 and
/// cannot connect, but the Wayland compositor socket being alive is a strong
/// signal that a live user session exists.  We therefore report available and
/// let the session-open path fail gracefully if screen capture is ultimately
/// not possible (e.g. compositor lacks the screencopy protocol).
fn is_rd_available() -> bool {
    use crate::client_data::platform::{has_wayland_display, has_x11_display};

    // Fast path: no display at all (headless server) — skip the more
    // expensive probes and avoid noisy Enigo log output every 5 minutes.
    if !has_x11_display() && !has_wayland_display() {
        return false;
    }

    // On Linux and FreeBSD the screen capturer uses x11rb exclusively.
    // Try an actual connection — same call X11Capturer::new() makes — so we
    // know capture will work before reporting available.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        if x11rb::connect(None).is_ok() {
            // X11 display is connectable; also verify input injection.
            use enigo::{Enigo, Settings};
            return Enigo::new(&Settings::default()).is_ok();
        }

        // X11 connect failed (no X11 server, or XWayland auth not available).
        // Check the Wayland compositor socket — WaylandCapturer::new() will
        // verify wlr-screencopy protocol support at actual session-open time.
        return has_wayland_display();
    }

    // All other platforms: use Enigo as the ground-truth probe.
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        use enigo::{Enigo, Settings};
        Enigo::new(&Settings::default()).is_ok()
    }
}
