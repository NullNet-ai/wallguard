import { useSearchParams, useParams, Link } from 'react-router-dom'
import { Terminal } from '@/components/Terminal'
import { RemoteDesktop } from '@/components/RemoteDesktop'

export function DeviceTunnels() {
  const { id = '' } = useParams()
  const [params] = useSearchParams()

  const session = params.get('session')
  const type    = params.get('type') as 'ssh' | 'tty' | 'rdp' | null
  const ws      = params.get('ws')
  const w       = Number(params.get('w') ?? 1280)
  const h       = Number(params.get('h') ?? 720)

  if (session && ws && type) {
    return (
      <div className="flex h-[calc(100vh-5rem)] flex-col gap-4">
        <div className="flex items-center gap-4">
          <Link to={`/devices/${id}`} className="text-sm text-gray-500 hover:underline">← Device</Link>
          <span className="text-sm text-gray-400 uppercase">{type} session</span>
        </div>
        <div className="flex-1">
          {type === 'rdp' ? (
            <RemoteDesktop wsUrl={ws} width={w} height={h} />
          ) : (
            <Terminal wsUrl={ws} />
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to={`/devices/${id}`} className="text-sm text-gray-500 hover:underline">← Device</Link>
      </div>
      <p className="text-gray-400">No active session. Open a tunnel from the device page.</p>
    </div>
  )
}
