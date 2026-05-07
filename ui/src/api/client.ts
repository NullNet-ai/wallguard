import { useAuthStore } from '@/store/auth'

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message)
  }
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const token = useAuthStore.getState().token
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (token) headers['Authorization'] = `Bearer ${token}`

  const res = await fetch(path, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  })

  if (!res.ok) {
    let code = 'UNKNOWN'
    let message = res.statusText
    try {
      const json = await res.json()
      code = json.error?.code ?? code
      message = json.error?.message ?? message
    } catch {}
    throw new ApiError(res.status, code, message)
  }

  if (res.status === 204) return undefined as T
  return res.json()
}

export const get  = <T>(path: string)              => request<T>('GET',    path)
export const post = <T>(path: string, body: unknown) => request<T>('POST',   path, body)
export const del  = <T>(path: string)              => request<T>('DELETE', path)
