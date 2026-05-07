import { post } from './client'
import type { TunnelCreatedResponse } from '@/types/tunnel'

export const openSsh = (deviceId: string) =>
  post<TunnelCreatedResponse>(`/api/v1/devices/${deviceId}/tunnels/ssh`, {})

export const openTty = (deviceId: string) =>
  post<TunnelCreatedResponse>(`/api/v1/devices/${deviceId}/tunnels/tty`, {})

export const openHttp = (deviceId: string, target_host: string, target_port: number) =>
  post<TunnelCreatedResponse>(`/api/v1/devices/${deviceId}/tunnels/http`, { target_host, target_port })

export const openRdp = (
  deviceId: string,
  width: number,
  height: number,
  target_fps: number,
  target_kbps: number,
) =>
  post<TunnelCreatedResponse>(`/api/v1/devices/${deviceId}/tunnels/rdp`, {
    width,
    height,
    target_fps,
    target_kbps,
  })
