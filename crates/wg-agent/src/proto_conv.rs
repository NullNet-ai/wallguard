use wg_shared::types::{FailureCategory, FailureSeverity, Feature, FirewallKind};

use crate::config::Config;
use crate::failure_buffer::FailureEntry;
use crate::proto::control::{
    client_message,
    AgentFailure as ProtoFailure,
    ClientMessage, CommandResult, CommandStatus,
    FailureCategory as ProtoCategory,
    FailureSeverity as ProtoSeverity,
    Feature as ProtoFeature,
    FirewallKind as ProtoFirewallKind,
    Hello,
};

pub fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn make_hello(features: &[Feature], config: &Config) -> ClientMessage {
    ClientMessage {
        message: Some(client_message::Message::Hello(Hello {
            protocol_version:       wg_shared::capabilities::PROTOCOL_VERSION,
            min_compatible_version: wg_shared::capabilities::MIN_AGENT_PROTOCOL_VERSION,
            supported_features:     features.iter().map(|&f| shared_to_proto_feature(f)).collect(),
            agent_version:          env!("CARGO_PKG_VERSION").to_string(),
            firewall_kind:          firewall_to_proto(config.device.firewall_kind),
        })),
    }
}

pub fn cmd_result(command_id: &str, status: CommandStatus, error_msg: &str) -> ClientMessage {
    ClientMessage {
        message: Some(client_message::Message::CommandResult(CommandResult {
            command_id:         command_id.to_string(),
            status:             status as i32,
            error_message:      error_msg.to_string(),
            applied_digest:     String::new(),
            output:             String::new(),
            applied_at_unix_ms: unix_ms_now(),
        })),
    }
}

pub fn failure_entry_to_proto(e: &FailureEntry, is_replay: bool) -> ProtoFailure {
    ProtoFailure {
        failure_id:  e.failure_id.to_string(),
        severity:    severity_to_proto(e.severity),
        category:    category_to_proto(e.category),
        message:     e.message.clone(),
        context:     e.context.clone().unwrap_or_default(),
        occurred_at: e.occurred_at,
        is_replay,
    }
}

pub fn shared_to_proto_feature(f: Feature) -> i32 {
    match f {
        Feature::NetworkMonitoring   => ProtoFeature::NetworkMonitoring   as i32,
        Feature::TelemetryMonitoring => ProtoFeature::TelemetryMonitoring as i32,
        Feature::ConfigMonitoring    => ProtoFeature::ConfigMonitoring    as i32,
        Feature::SshTunnel           => ProtoFeature::SshTunnel           as i32,
        Feature::TtyTunnel           => ProtoFeature::TtyTunnel           as i32,
        Feature::HttpTunnel          => ProtoFeature::HttpTunnel          as i32,
        Feature::NamedCommands       => ProtoFeature::NamedCommands       as i32,
        Feature::RemoteDesktop       => ProtoFeature::RemoteDesktop       as i32,
    }
}

pub fn proto_to_shared_feature(i: i32) -> Option<Feature> {
    match ProtoFeature::try_from(i).ok()? {
        ProtoFeature::NetworkMonitoring   => Some(Feature::NetworkMonitoring),
        ProtoFeature::TelemetryMonitoring => Some(Feature::TelemetryMonitoring),
        ProtoFeature::ConfigMonitoring    => Some(Feature::ConfigMonitoring),
        ProtoFeature::SshTunnel           => Some(Feature::SshTunnel),
        ProtoFeature::TtyTunnel           => Some(Feature::TtyTunnel),
        ProtoFeature::HttpTunnel          => Some(Feature::HttpTunnel),
        ProtoFeature::NamedCommands       => Some(Feature::NamedCommands),
        ProtoFeature::RemoteDesktop       => Some(Feature::RemoteDesktop),
    }
}

pub fn firewall_to_proto(k: FirewallKind) -> i32 {
    match k {
        FirewallKind::None     => ProtoFirewallKind::None     as i32,
        FirewallKind::PfSense  => ProtoFirewallKind::Pfsense  as i32,
        FirewallKind::OPNSense => ProtoFirewallKind::Opnsense as i32,
        FirewallKind::NFTables => ProtoFirewallKind::Nftables as i32,
    }
}

fn severity_to_proto(s: FailureSeverity) -> i32 {
    match s {
        FailureSeverity::Warning => ProtoSeverity::Warning as i32,
        FailureSeverity::Error   => ProtoSeverity::Error   as i32,
        FailureSeverity::Fatal   => ProtoSeverity::Fatal   as i32,
    }
}

fn category_to_proto(c: FailureCategory) -> i32 {
    match c {
        FailureCategory::Monitoring   => ProtoCategory::Monitoring   as i32,
        FailureCategory::Tunnel       => ProtoCategory::Tunnel       as i32,
        FailureCategory::DiskBuffer   => ProtoCategory::DiskBuffer   as i32,
        FailureCategory::Fireparse    => ProtoCategory::Fireparse     as i32,
        FailureCategory::AgentCrash   => ProtoCategory::AgentCrash   as i32,
        FailureCategory::Connectivity => ProtoCategory::Connectivity as i32,
        FailureCategory::System       => ProtoCategory::System       as i32,
    }
}
