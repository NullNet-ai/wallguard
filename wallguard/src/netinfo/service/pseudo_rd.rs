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
/// We use `Enigo::new()` as the ground-truth probe because it exercises the
/// same code path as the actual Remote Desktop feature (mouse/keyboard
/// injection).  A socket existing on disk is not sufficient — enigo also
/// needs authenticated access, which is only available while a user session
/// is running.
fn is_rd_available() -> bool {
    use enigo::{Enigo, Settings};

    // Suppress the enigo attempt entirely when there is no display socket at
    // all (e.g. a headless server).  Skipping the attempt avoids enigo's
    // internal error logs appearing every 5 minutes for machines that never
    // had a display.
    if !crate::client_data::platform::has_desktop_environment() {
        return false;
    }

    Enigo::new(&Settings::default()).is_ok()
}
