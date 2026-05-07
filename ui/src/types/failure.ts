export type FailureSeverity = 'Warning' | 'Error' | 'Fatal'

export type FailureCategory =
  | 'Monitoring'
  | 'Tunnel'
  | 'DiskBuffer'
  | 'Fireparse'
  | 'AgentCrash'
  | 'Connectivity'
  | 'System'

export interface AgentFailure {
  failure_id: string
  device_id: string
  severity: FailureSeverity
  category: FailureCategory
  message: string
  context: Record<string, string> | null
  occurred_at: string
  received_at: string
  is_replay: boolean
}
