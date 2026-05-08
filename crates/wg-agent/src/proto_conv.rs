use sysinfo::{Disks, Networks, System};
use wg_shared::types::{FailureCategory, FailureSeverity, Feature, FirewallKind};

use crate::failure_buffer::FailureEntry;
use crate::proto::control::{
    client_message,
    AgentFailure as ProtoFailure,
    ClientMessage, CommandResult, CommandStatus,
    DiskInfo as ProtoDisk,
    FailureCategory as ProtoCategory,
    FailureSeverity as ProtoSeverity,
    Feature as ProtoFeature,
    FirewallKind as ProtoFirewallKind,
    Hello,
    NetInterface as ProtoIface,
    SystemInfo as ProtoSystemInfo,
};

pub fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn make_hello(features: &[Feature], firewall_kind: FirewallKind) -> ClientMessage {
    ClientMessage {
        message: Some(client_message::Message::Hello(Hello {
            protocol_version:       wg_shared::capabilities::PROTOCOL_VERSION,
            min_compatible_version: wg_shared::capabilities::MIN_AGENT_PROTOCOL_VERSION,
            supported_features:     features.iter().map(|&f| shared_to_proto_feature(f)).collect(),
            agent_version:          env!("CARGO_PKG_VERSION").to_string(),
            firewall_kind:          firewall_to_proto(firewall_kind),
            system_info:            Some(collect_system_info()),
        })),
    }
}

fn collect_system_info() -> ProtoSystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_brand = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
    let cpu_cores = sys.physical_core_count().unwrap_or_else(|| sys.cpus().len()) as u32;

    let disks = Disks::new_with_refreshed_list()
        .iter()
        .map(|d| ProtoDisk {
            name:        d.name().to_string_lossy().into_owned(),
            total_bytes: d.total_space(),
        })
        .collect();

    let interfaces = Networks::new_with_refreshed_list()
        .iter()
        .map(|(name, data)| ProtoIface {
            name: name.clone(),
            mac:  data.mac_address().to_string(),
        })
        .collect();

    ProtoSystemInfo {
        hostname:        System::host_name().unwrap_or_default(),
        os_name:         System::name().unwrap_or_default(),
        os_version:      System::os_version().unwrap_or_default(),
        kernel_version:  System::kernel_version().unwrap_or_default(),
        arch:            System::cpu_arch().unwrap_or_else(|| std::env::consts::ARCH.to_string()),
        cpu_brand,
        cpu_cores,
        total_mem_bytes: sys.total_memory(),
        disks,
        interfaces,
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
        FirewallKind::IPTables => ProtoFirewallKind::Iptables as i32,
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
