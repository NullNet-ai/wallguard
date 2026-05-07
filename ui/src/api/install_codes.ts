import { get, post } from './client'
import type { PaginatedResponse } from '@/types/api'
import type { InstallationCode } from '@/types/install_code'

export const listInstallCodes = () =>
  get<PaginatedResponse<InstallationCode>>('/api/v1/installation-codes')

export const createInstallCode = (ttl_hours?: number) =>
  post<InstallationCode>('/api/v1/installation-codes', { ttl_hours })
