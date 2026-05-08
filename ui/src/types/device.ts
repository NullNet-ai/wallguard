export type FirewallKind = 'PfSense' | 'OPNSense' | 'NFTables' | 'None'

export interface DiskInfo {
  name:        string
  total_bytes: number
}

export interface NetInterface {
  name: string
  mac:  string
}

export interface SystemInfo {
  hostname:        string
  os_name:         string
  os_version:      string
  kernel_version:  string
  arch:            string
  cpu_brand:       string
  cpu_cores:       number
  total_mem_bytes: number
  disks:           DiskInfo[]
  interfaces:      NetInterface[]
}

export type Feature =
  | 'network_monitoring'
  | 'telemetry_monitoring'
  | 'config_monitoring'
  | 'ssh_tunnel'
  | 'tty_tunnel'
  | 'http_tunnel'
  | 'named_commands'
  | 'remote_desktop'

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
  system_info: SystemInfo | null
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
