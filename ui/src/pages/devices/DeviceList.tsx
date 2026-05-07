import { useQuery } from '@tanstack/react-query'
import { DeviceCard } from '@/components/DeviceCard'
import { listDevices, getDeviceStatus } from '@/api/devices'

export function DeviceList() {
  const { data, isLoading } = useQuery({
    queryKey: ['devices'],
    queryFn: listDevices,
  })

  if (isLoading) return <p className="text-gray-400">Loading…</p>

  const devices = data?.items ?? []

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-slate-900">Devices</h1>
      {devices.length === 0 ? (
        <p className="text-gray-400">No devices enrolled yet.</p>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {devices.map((device) => (
            <DeviceCardWithStatus key={device.id} device={device} />
          ))}
        </div>
      )}
    </div>
  )
}

function DeviceCardWithStatus({ device }: { device: import('@/types/device').Device }) {
  const { data: status } = useQuery({
    queryKey: ['device-status', device.id],
    queryFn: () => getDeviceStatus(device.id),
    refetchInterval: 30_000,
  })
  return (
    <DeviceCard
      device={device}
      connected={status?.connected ?? false}
      degraded={status?.degraded}
    />
  )
}
