// Domain types shared between wg-server and wg-ui.
// Hand-written; compiles to both native and wasm32.
// Proto-generated transport types live in each crate's build output.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id:         Uuid,
    pub name:       String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id:           Uuid,
    pub org_id:       Uuid,
    pub email:        String,
    pub display_name: String,
    pub role:         Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Admin,
    Operator,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id:            Uuid,
    pub org_id:        Uuid,
    pub display_name:  String,
    pub firewall_kind: FirewallKind,
    pub agent_version: Option<String>,
    pub features:      Vec<Feature>,
    pub notes:         Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewallKind {
    PfSense,
    OPNSense,
    NFTables,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_id:           Uuid,
    pub connected:           bool,
    pub degraded:            bool,
    pub active_tunnel_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelSession {
    pub id:          Uuid,
    pub device_id:   Uuid,
    pub tunnel_type: TunnelType,
    pub status:      TunnelStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFailure {
    pub failure_id:  Uuid,
    pub device_id:   Uuid,
    pub severity:    FailureSeverity,
    pub category:    FailureCategory,
    pub message:     String,
    pub context:     Option<serde_json::Value>,
    pub occurred_at: i64,
    pub is_replay:   bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FailureSeverity {
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Firewall rule types — populated in Phase 12.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id:        Uuid,
    pub device_id: Uuid,
    pub rule_type: FirewallRuleType,
    pub rule_data: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewallRuleType {
    Filter,
    Nat,
    Alias,
}
