import { get } from './client'
import type { TrafficResponse } from '@/types/traffic'

export function getDeviceTraffic(
  deviceId: string,
  window: string,
  bucket: string,
): Promise<TrafficResponse> {
  return get(`/api/v1/devices/${deviceId}/traffic?window=${window}&bucket=${bucket}`)
}
