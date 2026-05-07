import { Link } from 'react-router-dom'
import type { Device } from '@/types/device'
import { StatusBadge } from './StatusBadge'

interface Props {
  device: Device
  connected: boolean
  degraded?: boolean
}

export function DeviceCard({ device, connected, degraded }: Props) {
  return (
    <Link
      to={`/devices/${device.id}`}
      className="block rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition hover:shadow-md"
    >
      <div className="flex items-start justify-between gap-2">
        <h3 className="font-semibold text-slate-900 truncate">{device.display_name}</h3>
        <StatusBadge connected={connected} degraded={degraded} />
      </div>
      <div className="mt-3 space-y-1 text-sm text-gray-500">
        <p>{device.firewall_kind}</p>
        <p>v{device.agent_version}</p>
      </div>
    </Link>
  )
}
