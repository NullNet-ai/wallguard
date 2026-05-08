import { useState, useMemo } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { StatusBadge } from '@/components/StatusBadge'
import { TrafficChart } from '@/components/TrafficChart'
import { TelemetryDashboard } from '@/components/TelemetryDashboard'
import { useServerEvents } from '@/hooks/useServerEvents'
import { getDevice, getDeviceStatus, listHttpServices } from '@/api/devices'
import { openSsh, openTty, openRdp } from '@/api/tunnels'
import { useAuthStore } from '@/store/auth'
import { ApiError } from '@/api/client'

const RDP_W = 1280, RDP_H = 720, RDP_FPS = 30, RDP_KBPS = 4000

export function DeviceDetail() {
  const { id = '' } = useParams()
  const token = useAuthStore((s) => s.token)
  const qc = useQueryClient()
  const [tunnelError, setTunnelError] = useState<string | null>(null)

  const { data: device } = useQuery({ queryKey: ['device', id], queryFn: () => getDevice(id) })
  const { data: status } = useQuery({
    queryKey: ['device-status', id],
    queryFn: () => getDeviceStatus(id),
    refetchInterval: 30_000,
  })
  const { data: services } = useQuery({
    queryKey: ['http-services', id],
    queryFn: () => listHttpServices(id),
  })

  useServerEvents('http_services_updated', (e) => {
    if ((e.data as string).includes(id)) {
      void qc.invalidateQueries({ queryKey: ['http-services', id] })
    }
  })

  const withToken = (url: string) =>
    token ? `${url}?token=${encodeURIComponent(token)}` : url

  const handleSsh = async () => {
    try {
      const r = await openSsh(id)
      window.location.href = `/devices/${id}/tunnels?session=${r.session_id}&type=ssh&ws=${encodeURIComponent(withToken(r.ws_url))}`
    } catch (e) { setTunnelError(e instanceof ApiError ? e.message : 'Failed') }
  }

  const handleTty = async () => {
    try {
      const r = await openTty(id)
      window.location.href = `/devices/${id}/tunnels?session=${r.session_id}&type=tty&ws=${encodeURIComponent(withToken(r.ws_url))}`
    } catch (e) { setTunnelError(e instanceof ApiError ? e.message : 'Failed') }
  }

  const handleRdp = async () => {
    try {
      const r = await openRdp(id, RDP_W, RDP_H, RDP_FPS, RDP_KBPS)
      window.location.href = `/devices/${id}/tunnels?session=${r.session_id}&type=rdp&w=${RDP_W}&h=${RDP_H}&ws=${encodeURIComponent(withToken(r.ws_url))}`
    } catch (e) { setTunnelError(e instanceof ApiError ? e.message : 'Failed') }
  }

  const fmtBytes = useMemo(() => (b: number) => {
    if (b >= 1_073_741_824) return `${(b / 1_073_741_824).toFixed(1)} GB`
    if (b >= 1_048_576)     return `${(b / 1_048_576).toFixed(1)} MB`
    return `${(b / 1_024).toFixed(0)} KB`
  }, [])

  if (!device) return <p className="text-gray-400">Loading…</p>

  const features = device.features ?? []
  const hasSsh = features.includes('ssh_tunnel')
  const hasTty = features.includes('tty_tunnel')
  const hasRdp = features.includes('remote_desktop')
  const si = device.system_info

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to="/devices" className="text-sm text-gray-500 hover:underline">← Devices</Link>
      </div>

      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-slate-900">{device.display_name}</h1>
        {status && <StatusBadge connected={status.connected} degraded={status.degraded} />}
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div className="col-span-2 space-y-6">
          {/* Metadata */}
          <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
            <h2 className="mb-4 font-semibold text-slate-900">Details</h2>
            <dl className="grid grid-cols-2 gap-3 text-sm">
              <dt className="text-gray-500">Firewall</dt>
              <dd className="text-slate-900">{device.firewall_kind}</dd>
              <dt className="text-gray-500">Agent version</dt>
              <dd className="text-slate-900">v{device.agent_version}</dd>
              <dt className="text-gray-500">Enrolled</dt>
              <dd className="text-slate-900">{new Date(device.enrolled_at).toLocaleDateString()}</dd>
              <dt className="text-gray-500">Last seen</dt>
              <dd className="text-slate-900">{device.last_seen_at ? new Date(device.last_seen_at).toLocaleString() : '—'}</dd>
              <dt className="text-gray-500">Features</dt>
              <dd className="text-slate-900">{features.join(', ') || '—'}</dd>
            </dl>

            {si && (
              <>
                <hr className="my-4 border-gray-100" />
                <h3 className="mb-3 text-sm font-semibold text-slate-700">System</h3>
                <dl className="grid grid-cols-2 gap-3 text-sm">
                  {si.hostname       && <><dt className="text-gray-500">Hostname</dt>       <dd className="text-slate-900">{si.hostname}</dd></>}
                  {si.os_name        && <><dt className="text-gray-500">OS</dt>             <dd className="text-slate-900">{si.os_name} {si.os_version}</dd></>}
                  {si.kernel_version && <><dt className="text-gray-500">Kernel</dt>         <dd className="font-mono text-xs text-slate-900">{si.kernel_version}</dd></>}
                  {si.arch           && <><dt className="text-gray-500">Architecture</dt>   <dd className="text-slate-900">{si.arch}</dd></>}
                  {si.cpu_brand      && <><dt className="text-gray-500">CPU</dt>            <dd className="text-slate-900">{si.cpu_brand} ({si.cpu_cores} cores)</dd></>}
                  {si.total_mem_bytes > 0 && <><dt className="text-gray-500">Memory</dt>   <dd className="text-slate-900">{fmtBytes(si.total_mem_bytes)}</dd></>}
                </dl>

                {si.disks.length > 0 && (
                  <>
                    <h3 className="mb-2 mt-4 text-sm font-semibold text-slate-700">Storage</h3>
                    <div className="space-y-1">
                      {si.disks.map((d) => (
                        <div key={d.name} className="flex items-center justify-between rounded-md bg-gray-50 px-3 py-1.5 text-sm">
                          <span className="font-mono text-xs text-slate-700">{d.name}</span>
                          <span className="text-gray-500">{fmtBytes(d.total_bytes)}</span>
                        </div>
                      ))}
                    </div>
                  </>
                )}

                {si.interfaces.length > 0 && (
                  <>
                    <h3 className="mb-2 mt-4 text-sm font-semibold text-slate-700">Network Interfaces</h3>
                    <div className="space-y-1">
                      {si.interfaces.map((iface) => (
                        <div key={iface.name} className="flex items-center justify-between rounded-md bg-gray-50 px-3 py-1.5 text-sm">
                          <span className="font-mono text-xs text-slate-700">{iface.name}</span>
                          <span className="font-mono text-xs text-gray-400">{iface.mac}</span>
                        </div>
                      ))}
                    </div>
                  </>
                )}
              </>
            )}

            {device.notes && (
              <p className="mt-4 rounded-md bg-gray-50 p-3 text-sm text-gray-600">{device.notes}</p>
            )}
          </div>

          {/* Traffic chart */}
          <TrafficChart deviceId={id} />

          {/* Telemetry */}
          <TelemetryDashboard deviceId={id} />

          {/* HTTP Services */}
          {(services?.length ?? 0) > 0 && (
            <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
              <h2 className="mb-4 font-semibold text-slate-900">HTTP Services</h2>
              <div className="space-y-2">
                {services!.map((svc) => (
                  <div key={svc.port} className="flex items-center justify-between rounded-md bg-gray-50 px-4 py-2">
                    <span className="text-sm font-medium">{svc.title || `${svc.scheme}:${svc.port}`}</span>
                    <a
                      href={`/devices/${id}/tunnels/http/${svc.port}`}
                      className="text-sm font-medium text-blue-600 hover:underline"
                    >
                      Open →
                    </a>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="space-y-3">
          <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
            <h2 className="mb-4 font-semibold text-slate-900">Tunnels</h2>
            <div className="space-y-2">
              {hasSsh && (
                <button onClick={handleSsh} className="w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
                  Open SSH
                </button>
              )}
              {hasTty && (
                <button onClick={handleTty} className="w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
                  Open TTY
                </button>
              )}
              {hasRdp && (
                <button onClick={handleRdp} className="w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700">
                  Remote Desktop
                </button>
              )}
              {!hasSsh && !hasTty && !hasRdp && (
                <p className="text-sm text-gray-400">No tunnel capabilities advertised</p>
              )}
            </div>
            {tunnelError && (
              <p className="mt-3 rounded-md bg-red-50 px-3 py-2 text-sm text-red-600">{tunnelError}</p>
            )}
          </div>
          <Link
            to={`/devices/${id}/failures`}
            className="block rounded-xl border border-gray-200 bg-white px-6 py-4 text-sm font-medium text-slate-900 shadow-sm hover:bg-gray-50"
          >
            View Failures →
          </Link>
        </div>
      </div>
    </div>
  )
}
