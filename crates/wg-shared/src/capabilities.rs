use crate::types::{Feature, FirewallKind};

/// Wire protocol version this build speaks.
/// Agents with `min_compatible_version > SERVER_PROTOCOL_VERSION` are rejected.
pub const PROTOCOL_VERSION: u32 = 2;

/// Minimum agent protocol version the server will accept.
/// Agents below this receive `VersionRejected` and must be upgraded manually.
pub const MIN_AGENT_PROTOCOL_VERSION: u32 = 2;

/// Derive the full capability set for an agent.
///
/// `remote_desktop_available` is the result of the runtime display-backend
/// probe performed at agent startup.  Passing `false` omits
/// `Feature::RemoteDesktop` regardless of platform or compile-time features.
/// This decouples capability advertisement from OS/compile-time guards,
/// allowing FreeBSD agents with an active X display to advertise the feature.
pub fn derive_capabilities(
    firewall: FirewallKind,
    remote_desktop_available: bool,
) -> Vec<Feature> {
    let mut caps = vec![
        Feature::NetworkMonitoring,
        Feature::TelemetryMonitoring,
        Feature::SshTunnel,
        Feature::TtyTunnel,
        Feature::HttpTunnel,
        Feature::NamedCommands,
    ];

    if firewall != FirewallKind::None {
        caps.push(Feature::ConfigMonitoring);
    }

    if remote_desktop_available {
        caps.push(Feature::RemoteDesktop);
    }

    caps
}

/// Negotiate features: intersection of what the agent supports and what the
/// server permits.  The server can restrict features per-device in the DB;
/// `server_permitted` represents that filtered set.
pub fn negotiate(
    agent_supported: &[Feature],
    server_permitted: &[Feature],
) -> Vec<Feature> {
    agent_supported
        .iter()
        .filter(|f| server_permitted.contains(f))
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Feature, FirewallKind};

    #[test]
    fn base_features_always_present() {
        for kind in [
            FirewallKind::PfSense,
            FirewallKind::OPNSense,
            FirewallKind::NFTables,
            FirewallKind::None,
        ] {
            let caps = derive_capabilities(kind, false);
            for f in [
                Feature::NetworkMonitoring,
                Feature::TelemetryMonitoring,
                Feature::SshTunnel,
                Feature::TtyTunnel,
                Feature::HttpTunnel,
                Feature::NamedCommands,
            ] {
                assert!(caps.contains(&f), "{kind:?} missing {f:?}");
            }
        }
    }

    #[test]
    fn config_monitoring_only_with_firewall() {
        assert!(!derive_capabilities(FirewallKind::None, false)
            .contains(&Feature::ConfigMonitoring));
        for kind in [FirewallKind::PfSense, FirewallKind::OPNSense, FirewallKind::NFTables] {
            assert!(derive_capabilities(kind, false).contains(&Feature::ConfigMonitoring),
                "{kind:?} should have ConfigMonitoring");
        }
    }

    #[test]
    fn remote_desktop_controlled_by_runtime_probe() {
        let without = derive_capabilities(FirewallKind::None, false);
        let with_   = derive_capabilities(FirewallKind::None, true);

        assert!(!without.contains(&Feature::RemoteDesktop),
            "RemoteDesktop must not appear when probe returns false");
        assert!(with_.contains(&Feature::RemoteDesktop),
            "RemoteDesktop must appear when probe returns true");
    }

    #[test]
    fn remote_desktop_works_on_any_firewall_kind_if_probe_succeeds() {
        // FreeBSD + pfSense with a display should be able to use remote desktop.
        for kind in [FirewallKind::PfSense, FirewallKind::OPNSense, FirewallKind::NFTables, FirewallKind::None] {
            let caps = derive_capabilities(kind, true);
            assert!(caps.contains(&Feature::RemoteDesktop), "{kind:?} should have RemoteDesktop when probe passes");
        }
    }

    #[test]
    fn negotiate_returns_intersection() {
        let agent   = vec![Feature::SshTunnel, Feature::TtyTunnel, Feature::RemoteDesktop];
        let server  = vec![Feature::SshTunnel, Feature::NetworkMonitoring];
        let result  = negotiate(&agent, &server);
        assert_eq!(result, vec![Feature::SshTunnel]);
    }

    #[test]
    fn negotiate_empty_when_no_overlap() {
        let result = negotiate(&[Feature::SshTunnel], &[Feature::TtyTunnel]);
        assert!(result.is_empty());
    }
}
