export interface TrafficPoint {
  t: number        // unix ms
  out_bytes: number
  in_bytes: number
}

export interface TrafficResponse {
  points: TrafficPoint[]
  window: string
  bucket: string
}
