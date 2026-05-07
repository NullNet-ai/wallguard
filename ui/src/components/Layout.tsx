import { Link, useLocation } from 'react-router-dom'
import { useAuth } from '@/hooks/useAuth'

interface Props {
  children: React.ReactNode
}

const navLinks = [
  { to: '/',                      label: 'Dashboard' },
  { to: '/devices',               label: 'Devices' },
  { to: '/settings/users',        label: 'Users' },
  { to: '/settings/install-codes', label: 'Install Codes' },
]

export function Layout({ children }: Props) {
  const { logout } = useAuth()
  const { pathname } = useLocation()

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-slate-900 text-white">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-3">
          <div className="flex items-center gap-8">
            <span className="text-lg font-bold tracking-tight">WallGuard</span>
            <nav className="flex gap-1">
              {navLinks.map(({ to, label }) => {
                const active = to === '/'
                  ? pathname === '/'
                  : pathname.startsWith(to)
                return (
                  <Link
                    key={to}
                    to={to}
                    className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
                      active
                        ? 'bg-white/10 text-white'
                        : 'text-slate-300 hover:bg-white/5 hover:text-white'
                    }`}
                  >
                    {label}
                  </Link>
                )
              })}
            </nav>
          </div>
          <button
            onClick={logout}
            className="rounded-md px-3 py-1.5 text-sm font-medium text-slate-300 hover:bg-white/5 hover:text-white"
          >
            Logout
          </button>
        </div>
      </header>
      <main className="mx-auto max-w-7xl px-4 py-8">{children}</main>
    </div>
  )
}
