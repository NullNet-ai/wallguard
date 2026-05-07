export interface PaginatedResponse<T> {
  items: T[]
  total: number
}

export interface ApiError {
  code: string
  message: string
}
