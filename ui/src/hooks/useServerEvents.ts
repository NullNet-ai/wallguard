import { useEffect, useRef } from 'react'
import { useAuthStore } from '@/store/auth'

type Handler = (event: MessageEvent) => void

export function useServerEvents(
  eventType: string | string[] | null,
  handler: Handler,
) {
  const token = useAuthStore((s) => s.token)
  const handlerRef = useRef(handler)
  handlerRef.current = handler

  useEffect(() => {
    const url = token
      ? `/api/v1/events?token=${encodeURIComponent(token)}`
      : '/api/v1/events'

    const es = new EventSource(url)
    const cb = (e: MessageEvent) => handlerRef.current(e)

    const types = Array.isArray(eventType) ? eventType : eventType ? [eventType] : []

    if (types.length > 0) {
      types.forEach(t => es.addEventListener(t, cb))
    } else {
      es.onmessage = cb
    }

    return () => {
      types.forEach(t => es.removeEventListener(t, cb))
      es.close()
    }
  }, [token, Array.isArray(eventType) ? eventType.join(',') : eventType])
}
