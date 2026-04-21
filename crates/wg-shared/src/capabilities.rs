use crate::types::{Feature, FirewallKind};

/// Derive the static capability set for a given firewall kind.
///
/// The `RemoteDesktop` feature is NOT included here — it is added at agent
/// startup only if the runtime display probe succeeds (see
/// `wg-agent/src/capabilities.rs`). This function returns the compile-time
/// base set that is always correct regardless of runtime environment.
pub fn base_capabilities(firewall: FirewallKind) -> Vec<Feature> {
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

    caps
}
