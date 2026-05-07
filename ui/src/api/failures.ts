import { get } from './client'
import type { PaginatedResponse } from '@/types/api'
import type { AgentFailure, FailureSeverity } from '@/types/failure'

export const listFailures = (
  deviceId: string,
  offset: number,
  limit: number,
  severity?: FailureSeverity,
) => {
  const params = new URLSearchParams({
    offset: String(offset),
    limit: String(limit),
  })
  if (severity) params.set('severity', severity)
  return get<PaginatedResponse<AgentFailure>>(
    `/api/v1/devices/${deviceId}/failures?${params}`,
  )
}

export const listOrgFailures = (offset: number, limit: number) => {
  const params = new URLSearchParams({ offset: String(offset), limit: String(limit) })
  return get<PaginatedResponse<AgentFailure>>(`/api/v1/failures?${params}`)
}
