export interface MetricsPoint {
  t: number
  cpu_percent: number
  mem_used_bytes: number
  mem_total_bytes: number
  disk_used_bytes: number
  disk_total_bytes: number
  load_1m: number
}

export interface MetricsResponse {
  points: MetricsPoint[]
  window: string
  bucket: string
}
