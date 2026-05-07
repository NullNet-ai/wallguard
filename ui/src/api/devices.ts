import { get } from './client'
import type { PaginatedResponse } from '@/types/api'
import type { Device, DeviceStatus, HttpService } from '@/types/device'

export const listDevices = () =>
  get<PaginatedResponse<Device>>('/api/v1/devices')

export const getDevice = (id: string) =>
  get<Device>(`/api/v1/devices/${id}`)

export const getDeviceStatus = (id: string) =>
  get<DeviceStatus>(`/api/v1/devices/${id}/status`)

export const listHttpServices = (deviceId: string) =>
  get<HttpService[]>(`/api/v1/devices/${deviceId}/http-services`)
