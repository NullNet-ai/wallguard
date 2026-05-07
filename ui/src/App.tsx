import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PrivateRoute } from '@/components/PrivateRoute'
import { Login } from '@/pages/Login'
import { Dashboard } from '@/pages/Dashboard'
import { DeviceList } from '@/pages/devices/DeviceList'
import { DeviceDetail } from '@/pages/devices/DeviceDetail'
import { DeviceFailures } from '@/pages/devices/DeviceFailures'
import { DeviceTunnels } from '@/pages/devices/DeviceTunnels'
import { Users } from '@/pages/settings/Users'
import { InstallCodes } from '@/pages/settings/InstallCodes'

const qc = new QueryClient({
  defaultOptions: { queries: { staleTime: 15_000, retry: 1 } },
})

export function App() {
  return (
    <QueryClientProvider client={qc}>
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route element={<PrivateRoute />}>
            <Route path="/"                           element={<Dashboard />} />
            <Route path="/devices"                    element={<DeviceList />} />
            <Route path="/devices/:id"                element={<DeviceDetail />} />
            <Route path="/devices/:id/failures"       element={<DeviceFailures />} />
            <Route path="/devices/:id/tunnels"        element={<DeviceTunnels />} />
            <Route path="/settings/users"             element={<Users />} />
            <Route path="/settings/install-codes"     element={<InstallCodes />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  )
}
