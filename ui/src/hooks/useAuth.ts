import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/store/auth'
import * as authApi from '@/api/auth'

export function useAuth() {
  const { token, setToken, clearToken } = useAuthStore()
  const navigate = useNavigate()

  const login = async (email: string, password: string) => {
    const res = await authApi.login(email, password)
    setToken(res.access_token)
    navigate('/')
  }

  const logout = async () => {
    try { await authApi.logout() } catch {}
    clearToken()
    navigate('/login')
  }

  return { token, isAuthed: !!token, login, logout }
}
