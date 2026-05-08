// Domain types shared between wg-server and wg-ui.
// Hand-written; must compile to both native and wasm32-unknown-unknown.
// Proto-generated transport types live in each crate's build output.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Organizations & Users
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id:         Uuid,
    pub name:       String,
    pub created_at: i64,  // Unix ms
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id:           Uuid,
    pub org_id:       Uuid,
    pub email:        String,
    pub display_name: String,
    pub role:         Role,
    pub created_at:   i64,
}

/// RBAC role. Higher variants have strictly more privileges than lower ones.
/// Owner > Admin > Operator > Viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Admin,
    Operator,
    Viewer,
}

impl Role {
    fn level(self) -> u8 {
        match self {
            Self::Owner    => 4,
            Self::Admin    => 3,
            Self::Operator => 2,
            Self::Viewer   => 1,
        }
    }

    /// Returns true if this role is at least as privileged as `required`.
    pub fn satisfies(self, required: Role) -> bool {
        self.level() >= required.level()
    }

    pub fn can_open_tunnel(self) -> bool      { self.satisfies(Role::Operator) }
    pub fn can_push_rules(self) -> bool       { self.satisfies(Role::Operator) }
    pub fn can_manage_devices(self) -> bool   { self.satisfies(Role::Admin) }
    pub fn can_manage_users(self) -> bool     { self.satisfies(Role::Admin) }
    pub fn can_delete_org(self) -> bool       { self == Role::Owner }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Role {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }
}

// ---------------------------------------------------------------------------
// Devices
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id:             Uuid,
    pub org_id:         Uuid,
    pub display_name:   String,
    pub firewall_kind:  FirewallKind,
    pub agent_version:  Option<String>,
    /// Negotiated feature set from the most recent Hello/Welcome exchange.
    pub features:       Vec<Feature>,
    pub enrolled_at:    i64,             // Unix ms
    pub last_seen_at:   Option<i64>,     // Unix ms; None if never connected
    /// SHA-256 of the last applied firewall config; None before first snapshot.
    pub config_digest:  Option<String>,
    pub notes:          Option<String>,
    pub system_info:    Option<SystemInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub hostname:        String,
    pub os_name:         String,
    pub os_version:      String,
    pub kernel_version:  String,
    pub arch:            String,
    pub cpu_brand:       String,
    pub cpu_cores:       u32,
    pub total_mem_bytes: u64,
    pub disks:           Vec<DiskInfo>,
    pub interfaces:      Vec<NetInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name:        String,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetInterface {
    pub name: String,
    pub mac:  String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewallKind {
    #[serde(rename = "pfsense")]
    PfSense,
    #[serde(rename = "opnsense")]
    OPNSense,
    #[serde(rename = "nftables")]
    NFTables,
    #[serde(rename = "iptables")]
    IPTables,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    NetworkMonitoring,
    TelemetryMonitoring,
    ConfigMonitoring,
    SshTunnel,
    TtyTunnel,
    HttpTunnel,
    NamedCommands,
    RemoteDesktop,
}

/// Live status snapshot for a single device — updated from heartbeat data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_id:           Uuid,
    pub connected:           bool,
    pub degraded:            bool,
    pub active_tunnel_count: u32,
    pub last_seen_at:        Option<i64>,  // Unix ms
    pub monitoring:          MonitoringStatus,
}

/// Agent-reported monitoring pipeline metrics — included in every heartbeat.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonitoringStatus {
    pub packet_queue_depth:    u32,
    pub disk_buffer_bytes:     u64,
    pub disk_buffer_max_bytes: u64,
    pub packets_dropped_total: u64,
    pub packets_sent_total:    u64,
    pub degraded:              bool,
    pub active_tunnel_count:   u32,
}

// ---------------------------------------------------------------------------
// Tunnel sessions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelSession {
    pub id:             Uuid,
    pub device_id:      Uuid,
    pub tunnel_type:    TunnelType,
    pub status:         TunnelStatus,
    pub initiated_by:   Option<Uuid>,  // user UUID; None for automated sessions
    pub started_at:     i64,           // Unix ms
    pub ended_at:       Option<i64>,   // Unix ms
    pub bytes_sent:     u64,
    pub bytes_received: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelType {
    Ssh,
    Tty,
    Http,
    RemoteDesktop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Active,
    Closed,
    Abandoned,
}

// ---------------------------------------------------------------------------
// Agent failures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFailure {
    pub failure_id:  Uuid,
    pub device_id:   Uuid,
    pub severity:    FailureSeverity,
    pub category:    FailureCategory,
    pub message:     String,
    pub context:     Option<serde_json::Value>,
    pub occurred_at: i64,   // Unix ms at time of occurrence (not delivery)
    pub received_at: Option<i64>,
    pub is_replay:   bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FailureSeverity {
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    Monitoring,
    Tunnel,
    DiskBuffer,
    Fireparse,
    AgentCrash,
    Connectivity,
    System,
}

// ---------------------------------------------------------------------------
// Installation codes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationCode {
    pub code:       String,
    pub org_id:     Uuid,
    pub created_by: Uuid,
    pub used_at:    Option<i64>,
    pub expires_at: i64,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Firewall rules (Phase 12)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id:             Uuid,
    pub device_id:      Uuid,
    pub rule_type:      FirewallRuleType,
    pub rule_data:      serde_json::Value,
    pub applied_digest: Option<String>,
    pub applied_at:     Option<i64>,
    pub created_by:     Option<Uuid>,
    pub created_at:     i64,
    pub deleted_at:     Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewallRuleType {
    Filter,
    Nat,
    Alias,
}

// ---------------------------------------------------------------------------
// Config drift
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDrift {
    pub device_id:        Uuid,
    pub expected_digest:  String,
    pub observed_digest:  String,
    pub detected_at:      i64,
}
