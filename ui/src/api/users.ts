import { get, post, del } from './client'
import type { PaginatedResponse } from '@/types/api'
import type { User, CreateUserRequest } from '@/types/user'

export const listUsers = () => get<PaginatedResponse<User>>('/api/v1/users')

export const createUser = (req: CreateUserRequest) => post<void>('/api/v1/users', req)

export const deleteUser = (id: string) => del<void>(`/api/v1/users/${id}`)
