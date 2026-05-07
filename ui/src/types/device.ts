export type FirewallKind = 'PfSense' | 'OPNSense' | 'NFTables' | 'None'

export type Feature =
  | 'NetworkMonitoring'
  | 'TelemetryMonitoring'
  | 'ConfigMonitoring'
  | 'SshTunnel'
  | 'TtyTunnel'
  | 'HttpTunnel'
  | 'NamedCommands'
  | 'RemoteDesktop'

export interface Device {
  id: string
  org_id: string
  display_name: string
  firewall_kind: FirewallKind
  agent_version: string
  features: Feature[]
  enrolled_at: string
  last_seen_at: string | null
  notes: string | null
}

export interface DeviceStatus {
  device_id: string
  connected: boolean
  degraded: boolean
  active_tunnel_count: number
  last_seen_at: string | null
}

export interface HttpService {
  port: number
  scheme: string
  title: string
}
