import { useEffect, useRef } from 'react'

interface Props {
  wsUrl: string
  width: number
  height: number
  id?: string
}

export function RemoteDesktop({ wsUrl, width, height, id = 'rdp-canvas' }: Props) {
  const mounted = useRef(false)

  useEffect(() => {
    if (mounted.current) return
    mounted.current = true
    window.wgRemoteDesktop.open(id, wsUrl, width, height)
    return () => { window.wgRemoteDesktop.dispose(id) }
  }, [id, wsUrl, width, height])

  return (
    <div className="flex flex-col items-center gap-3">
      <canvas
        id={id}
        width={width}
        height={height}
        className="max-w-full rounded-lg border border-gray-700 shadow-lg"
        style={{ aspectRatio: `${width}/${height}` }}
      />
      <button
        onClick={() => window.wgRemoteDesktop.sendPli(id)}
        className="rounded-md bg-slate-700 px-3 py-1.5 text-xs font-medium text-white hover:bg-slate-600"
      >
        Request Keyframe
      </button>
    </div>
  )
}
