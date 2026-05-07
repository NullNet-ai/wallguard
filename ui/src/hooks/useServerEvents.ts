import { useEffect, useRef } from 'react'
import { useAuthStore } from '@/store/auth'

type Handler = (event: MessageEvent) => void

export function useServerEvents(
  eventType: string | null,
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

    if (eventType) {
      es.addEventListener(eventType, cb)
    } else {
      es.onmessage = cb
    }

    return () => {
      if (eventType) {
        es.removeEventListener(eventType, cb)
      }
      es.close()
    }
  }, [token, eventType])
}
