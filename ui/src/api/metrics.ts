import { get } from './client'
import type { MetricsResponse } from '@/types/metrics'

export function getDeviceMetrics(
  deviceId: string,
  window: string,
  bucket: string,
): Promise<MetricsResponse> {
  return get(`/api/v1/devices/${deviceId}/metrics?window=${window}&bucket=${bucket}`)
}
