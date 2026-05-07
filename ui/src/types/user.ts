export type Role = 'Owner' | 'Admin' | 'Operator' | 'Viewer'

export interface User {
  id: string
  email: string
  name: string
  role: Role
}

export interface CreateUserRequest {
  email: string
  name: string
  password: string
  role: Role
}
