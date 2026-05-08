use std::path::Path;

use tracing::info;
use wg_shared::types::FirewallKind;

/// Detect the local firewall platform by inspecting well-known filesystem
/// paths.  No shell commands are executed.
///
/// Detection order matters: pfSense/OPNsense checks come first because those
/// systems may also have nftables/iptables installed.  Linux checks are last.
pub fn detect_firewall_kind() -> FirewallKind {
    let kind = detect_inner();
    info!("firewall auto-detection: {kind:?}");
    kind
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn detect_inner() -> FirewallKind {
    // --- pfSense (FreeBSD-based) ---
    if exists("/etc/pfsense-release") {
        return FirewallKind::PfSense;
    }

    // --- OPNsense (FreeBSD-based) ---
    if exists("/usr/local/opnsense/version/core")
        || exists("/usr/local/sbin/pluginctl")
    {
        return FirewallKind::OPNSense;
    }

    // --- Linux: prefer nftables over iptables ---
    if exists("/usr/sbin/nft") || exists("/sbin/nft") {
        return FirewallKind::NFTables;
    }

    if exists("/usr/sbin/iptables") || exists("/sbin/iptables") {
        return FirewallKind::IPTables;
    }

    FirewallKind::None
}

#[cfg(test)]
mod tests {
    use super::detect_inner;

    #[test]
    fn returns_a_kind_without_panicking() {
        // Just verify it runs cleanly on whatever platform the tests run on.
        let _ = detect_inner();
    }
}
