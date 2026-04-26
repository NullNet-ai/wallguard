use std::path::{Path, PathBuf};

use serde::Deserialize;
use wg_shared::types::FirewallKind;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub device:        DeviceConfig,
    pub server:        ServerConfig,
    pub tls:           TlsConfig,
    #[serde(default)]
    pub agent:         AgentConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub transmission:  TransmissionConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceConfig {
    pub id:            String,
    pub firewall_kind: FirewallKind,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub name:      String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_quic_port")]
    pub quic_port: u16,
    #[serde(default = "default_tcp_port")]
    pub tcp_port:  u16,
}

fn default_grpc_port() -> u16 { 50052 }
fn default_quic_port() -> u16 { 7777 }
fn default_tcp_port()  -> u16 { 7778 }

#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    pub device_key:  PathBuf,
    pub device_cert: PathBuf,
    pub ca_cert:     PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_s: u64,
    #[serde(default = "default_reconnect_base")]
    pub reconnect_base_s:     u64,
    #[serde(default = "default_reconnect_max")]
    pub reconnect_max_s:      u64,
    /// Local SSH daemon port for SSH tunnel relay (default 22).
    #[serde(default = "default_ssh_port")]
    pub ssh_port:             u16,
    /// Shell spawned for TTY tunnels (default /bin/sh).
    #[serde(default = "default_tty_shell")]
    pub tty_shell:            String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_s: default_heartbeat_interval(),
            reconnect_base_s:     default_reconnect_base(),
            reconnect_max_s:      default_reconnect_max(),
            ssh_port:             default_ssh_port(),
            tty_shell:            default_tty_shell(),
        }
    }
}

fn default_heartbeat_interval() -> u64 { 10 }
fn default_reconnect_base()     -> u64 { 1 }
fn default_reconnect_max()      -> u64 { 300 }
fn default_ssh_port()           -> u16 { 22 }
fn default_tty_shell()          -> String { "/bin/sh".to_string() }

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ObservabilityConfig {
    /// 0 disables the Prometheus endpoint.
    #[serde(default)]
    pub metrics_port:  u16,
    /// "json" or "pretty"
    #[serde(default = "default_log_format")]
    pub log_format:    String,
    /// OTLP endpoint; empty string disables export.
    #[serde(default)]
    pub otlp_endpoint: String,
}

fn default_log_format() -> String { "pretty".to_string() }

#[derive(Debug, Clone, Deserialize)]
pub struct TransmissionConfig {
    #[serde(default = "default_disk_buffer_path")]
    pub disk_buffer_path:      PathBuf,
    /// Max disk buffer size in bytes. Default 256 MiB.
    #[serde(default = "default_disk_buffer_max")]
    pub disk_buffer_max_bytes: u64,
    /// Minimum free disk space before disk buffer writes are refused. Default 512 MiB.
    #[serde(default = "default_disk_min_free")]
    pub disk_min_free_bytes:   u64,
    /// In-memory packet capture queue depth. Default 50_000.
    #[serde(default = "default_packet_queue_depth")]
    pub packet_queue_depth:    usize,
}

impl Default for TransmissionConfig {
    fn default() -> Self {
        Self {
            disk_buffer_path:      default_disk_buffer_path(),
            disk_buffer_max_bytes: default_disk_buffer_max(),
            disk_min_free_bytes:   default_disk_min_free(),
            packet_queue_depth:    default_packet_queue_depth(),
        }
    }
}

fn default_disk_buffer_path()   -> PathBuf { PathBuf::from("/var/lib/wallguard/buffer") }
fn default_disk_buffer_max()    -> u64     { 256 * 1024 * 1024 }
fn default_disk_min_free()      -> u64     { 512 * 1024 * 1024 }
fn default_packet_queue_depth() -> usize   { 50_000 }

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
        toml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("cannot parse {}: {e}", path.display()))
    }

    /// gRPC control-plane endpoint (mTLS).
    pub fn grpc_endpoint(&self) -> String {
        format!("https://{}:{}", self.server.name, self.server.grpc_port)
    }

    pub fn cli_socket_path() -> &'static str {
        "/run/wallguard/agent.sock"
    }
}
