import { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { SeverityBadge } from '@/components/SeverityBadge'
import { Pagination } from '@/components/Pagination'
import { listFailures } from '@/api/failures'
import type { FailureSeverity } from '@/types/failure'

const PAGE = 20

const SEVERITIES: Array<{ label: string; value: FailureSeverity | '' }> = [
  { label: 'All',     value: '' },
  { label: 'Warning', value: 'Warning' },
  { label: 'Error',   value: 'Error' },
  { label: 'Fatal',   value: 'Fatal' },
]

export function DeviceFailures() {
  const { id = '' } = useParams()
  const [offset,   setOffset]   = useState(0)
  const [severity, setSeverity] = useState<FailureSeverity | ''>('')

  const { data } = useQuery({
    queryKey: ['failures', id, offset, severity],
    queryFn:  () => listFailures(id, offset, PAGE, severity || undefined),
  })

  const items = data?.items ?? []
  const total = data?.total ?? 0

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to={`/devices/${id}`} className="text-sm text-gray-500 hover:underline">← Device</Link>
      </div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-slate-900">Failures</h1>
        <select
          value={severity}
          onChange={(e) => { setSeverity(e.target.value as FailureSeverity | ''); setOffset(0) }}
          className="rounded-lg border border-gray-300 px-3 py-1.5 text-sm"
        >
          {SEVERITIES.map((s) => (
            <option key={s.value} value={s.value}>{s.label}</option>
          ))}
        </select>
      </div>

      <div className="rounded-xl border border-gray-200 bg-white shadow-sm">
        {items.length === 0 ? (
          <p className="px-6 py-8 text-center text-sm text-gray-400">No failures found</p>
        ) : (
          <ul className="divide-y divide-gray-100">
            {items.map((f) => (
              <li key={f.failure_id} className="px-6 py-4">
                <div className="flex items-start justify-between gap-4">
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <SeverityBadge severity={f.severity} />
                      <span className="text-xs text-gray-400">{f.category}</span>
                      {f.is_replay && (
                        <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-500">replay</span>
                      )}
                    </div>
                    <p className="text-sm text-slate-900">{f.message}</p>
                  </div>
                  <time className="shrink-0 text-xs text-gray-400">
                    {new Date(f.occurred_at).toLocaleString()}
                  </time>
                </div>
              </li>
            ))}
          </ul>
        )}
        {total > PAGE && (
          <div className="border-t border-gray-100 px-6 py-4">
            <Pagination
              offset={offset} limit={PAGE} total={total}
              onPrev={() => setOffset((o) => Math.max(0, o - PAGE))}
              onNext={() => setOffset((o) => o + PAGE)}
            />
          </div>
        )}
      </div>
    </div>
  )
}
