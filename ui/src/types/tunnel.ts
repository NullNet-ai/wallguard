export type TunnelType = 'ssh' | 'tty' | 'http' | 'rdp'
export type TunnelStatus = 'Active' | 'Closed' | 'Abandoned'

export interface TunnelCreatedResponse {
  session_id: string
  ws_url: string
}
