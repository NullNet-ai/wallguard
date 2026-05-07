import { useEffect, useRef } from 'react'

interface Props {
  wsUrl: string
  id?: string
}

export function Terminal({ wsUrl, id = 'terminal' }: Props) {
  const mounted = useRef(false)

  useEffect(() => {
    if (mounted.current) return
    mounted.current = true
    window.wgTerminal.open(id, wsUrl)
    return () => { window.wgTerminal.dispose(id) }
  }, [id, wsUrl])

  return (
    <div
      id={id}
      className="h-full w-full rounded-lg bg-black"
      style={{ minHeight: '400px' }}
    />
  )
}
