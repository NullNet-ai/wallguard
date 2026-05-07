import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { StatCard } from '@/components/StatCard'
import { StatusBadge } from '@/components/StatusBadge'
import { useServerEvents } from '@/hooks/useServerEvents'
import { listDevices, getDeviceStatus } from '@/api/devices'
import { listOrgFailures } from '@/api/failures'

const MAX_EVENTS = 50

export function Dashboard() {
  const [events, setEvents] = useState<string[]>([])

  const { data: devicesData } = useQuery({
    queryKey: ['devices'],
    queryFn: listDevices,
  })
  const { data: failuresData } = useQuery({
    queryKey: ['org-failures'],
    queryFn: () => listOrgFailures(0, 1),
  })

  useServerEvents(null, (e) => {
    setEvents((prev) => {
      const next = [e.data as string, ...prev]
      return next.slice(0, MAX_EVENTS)
    })
  })

  const devices = devicesData?.items ?? []

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-bold text-slate-900">Dashboard</h1>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <StatCard label="Total Devices"    value={devicesData?.total ?? '—'} />
        <StatCard label="Recent Failures"  value={failuresData?.total ?? '—'} />
        <StatCard label="Live Events"      value={events.length} />
      </div>

      <div className="rounded-xl border border-gray-200 bg-white shadow-sm">
        <div className="border-b border-gray-200 px-6 py-4">
          <h2 className="font-semibold text-slate-900">Recent Devices</h2>
        </div>
        <ul className="divide-y divide-gray-100">
          {devices.slice(0, 5).map((device) => (
            <DeviceRow key={device.id} deviceId={device.id} name={device.display_name} />
          ))}
          {devices.length === 0 && (
            <li className="px-6 py-8 text-center text-sm text-gray-400">No devices enrolled</li>
          )}
        </ul>
        {devices.length > 0 && (
          <div className="border-t border-gray-100 px-6 py-3">
            <Link to="/devices" className="text-sm font-medium text-blue-600 hover:underline">
              View all devices →
            </Link>
          </div>
        )}
      </div>

      <div className="rounded-xl border border-gray-200 bg-white shadow-sm">
        <div className="border-b border-gray-200 px-6 py-4">
          <h2 className="font-semibold text-slate-900">Live Events</h2>
        </div>
        <ul className="divide-y divide-gray-100 font-mono text-xs">
          {events.slice(0, 20).map((msg, i) => (
            <li key={i} className="px-6 py-2 text-gray-600">{msg}</li>
          ))}
          {events.length === 0 && (
            <li className="px-6 py-8 text-center text-sm text-gray-400">Waiting for events…</li>
          )}
        </ul>
      </div>
    </div>
  )
}

function DeviceRow({ deviceId, name }: { deviceId: string; name: string }) {
  const { data: status } = useQuery({
    queryKey: ['device-status', deviceId],
    queryFn: () => getDeviceStatus(deviceId),
    refetchInterval: 30_000,
  })

  return (
    <li className="flex items-center justify-between px-6 py-3">
      <Link to={`/devices/${deviceId}`} className="font-medium text-blue-600 hover:underline">
        {name}
      </Link>
      {status && <StatusBadge connected={status.connected} degraded={status.degraded} />}
    </li>
  )
}
