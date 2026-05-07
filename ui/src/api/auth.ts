import { post } from './client'
import type { TokenResponse } from '@/types/auth'

export const login = (email: string, password: string) =>
  post<TokenResponse>('/api/v1/auth/login', { email, password })

export const logout = () => post<void>('/api/v1/auth/logout', {})

export const refresh = (refresh_token: string) =>
  post<TokenResponse>('/api/v1/auth/refresh', { refresh_token })
