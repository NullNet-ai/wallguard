import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip,
  ReferenceLine, ResponsiveContainer,
} from 'recharts'
import { getDeviceTraffic } from '@/api/traffic'

// ---------------------------------------------------------------------------
// Window / bucket presets
// ---------------------------------------------------------------------------

const WINDOWS = [
  { label: 'Live', window: '2m',  bucket: '10s', live: true  },
  { label: '15m',  window: '15m', bucket: '30s', live: false },
  { label: '1h',   window: '1h',  bucket: '1m',  live: false },
  { label: '6h',   window: '6h',  bucket: '5m',  live: false },
  { label: '24h',  window: '24h', bucket: '15m', live: false },
  { label: '7d',   window: '7d',  bucket: '1h',  live: false },
]

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

function fmtBytes(bytes: number): string {
  const abs = Math.abs(bytes)
  if (abs >= 1_073_741_824) return `${(abs / 1_073_741_824).toFixed(1)} GB`
  if (abs >= 1_048_576)     return `${(abs / 1_048_576).toFixed(1)} MB`
  if (abs >= 1_024)         return `${(abs / 1_024).toFixed(1)} KB`
  return `${abs} B`
}

function fmtTime(ts: number, window: string): string {
  const d = new Date(ts)
  if (window === '7d')  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
  if (window === '24h') return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
  if (window === '2m')  return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
}

// ---------------------------------------------------------------------------
// Custom tooltip
// ---------------------------------------------------------------------------

function ChartTooltip({ active, payload, label }: {
  active?: boolean
  payload?: Array<{ name: string; value: number }>
  label?: number
}) {
  if (!active || !payload?.length) return null
  const out = payload.find((p) => p.name === 'out_bytes')?.value ?? 0
  const inNeg = payload.find((p) => p.name === 'in_neg')?.value ?? 0

  return (
    <div className="rounded-lg border border-gray-200 bg-white px-3 py-2 shadow-md text-xs">
      <p className="mb-1 font-medium text-gray-500">{label != null ? new Date(label).toLocaleString() : ''}</p>
      <p className="text-blue-600">↑ Out: {fmtBytes(out)}</p>
      <p className="text-orange-500">↓ In:  {fmtBytes(Math.abs(inNeg))}</p>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function TrafficChart({ deviceId }: { deviceId: string }) {
  const [preset, setPreset] = useState(WINDOWS[0])

  const { data, isLoading } = useQuery({
    queryKey: ['traffic', deviceId, preset.window, preset.bucket],
    queryFn: () => getDeviceTraffic(deviceId, preset.window, preset.bucket),
    refetchInterval: preset.live ? 1_000 : 30_000,
  })

  const chartData = useMemo(
    () =>
      (data?.points ?? []).map((p) => ({
        t:        p.t,
        out_bytes: p.out_bytes,
        in_neg:   -p.in_bytes,  // inverted so it renders below zero
      })),
    [data],
  )

  const yMax = useMemo(() => {
    if (!chartData.length) return 1024
    const maxOut = Math.max(...chartData.map((p) => p.out_bytes))
    const maxIn  = Math.max(...chartData.map((p) => Math.abs(p.in_neg)))
    return Math.max(maxOut, maxIn, 1024)
  }, [chartData])

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="font-semibold text-slate-900">Traffic</h2>
        <div className="flex gap-1">
          {WINDOWS.map((w) => (
            <button
              key={w.label}
              onClick={() => setPreset(w)}
              className={`flex items-center gap-1.5 rounded px-3 py-1 text-xs font-medium transition-colors ${
                preset.label === w.label
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-500 hover:bg-gray-100'
              }`}
            >
              {w.live && (
                <span className={`inline-block h-1.5 w-1.5 rounded-full ${
                  preset.label === w.label ? 'bg-white animate-pulse' : 'bg-red-500'
                }`} />
              )}
              {w.label}
            </button>
          ))}
        </div>
      </div>

      {isLoading ? (
        <div className="flex h-52 items-center justify-center text-sm text-gray-400">Loading…</div>
      ) : chartData.length === 0 ? (
        <div className="flex h-52 items-center justify-center text-sm text-gray-400">
          No traffic data in this window
        </div>
      ) : (
        <ResponsiveContainer width="100%" height={220}>
          <AreaChart data={chartData} margin={{ top: 4, right: 8, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="gradOut" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%"  stopColor="#2563eb" stopOpacity={0.25} />
                <stop offset="95%" stopColor="#2563eb" stopOpacity={0.02} />
              </linearGradient>
              <linearGradient id="gradIn" x1="0" y1="1" x2="0" y2="0">
                <stop offset="5%"  stopColor="#f97316" stopOpacity={0.25} />
                <stop offset="95%" stopColor="#f97316" stopOpacity={0.02} />
              </linearGradient>
            </defs>

            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />

            <XAxis
              dataKey="t"
              type="number"
              domain={['dataMin', 'dataMax']}
              scale="time"
              tickFormatter={(v) => fmtTime(v as number, preset.window)}
              tick={{ fontSize: 11, fill: '#94a3b8' }}
              tickLine={false}
              axisLine={false}
              minTickGap={40}
            />

            <YAxis
              domain={[-yMax, yMax]}
              tickFormatter={(v) => (v === 0 ? '0' : `${v > 0 ? '' : ''}${fmtBytes(v as number)}`)}
              tick={{ fontSize: 11, fill: '#94a3b8' }}
              tickLine={false}
              axisLine={false}
              width={64}
            />

            <Tooltip content={<ChartTooltip />} />

            <ReferenceLine y={0} stroke="#e2e8f0" strokeWidth={1} />

            <Area
              type="monotone"
              dataKey="out_bytes"
              name="out_bytes"
              stroke="#2563eb"
              strokeWidth={1.5}
              fill="url(#gradOut)"
              dot={false}
              isAnimationActive={false}
            />
            <Area
              type="monotone"
              dataKey="in_neg"
              name="in_neg"
              stroke="#f97316"
              strokeWidth={1.5}
              fill="url(#gradIn)"
              dot={false}
              isAnimationActive={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      )}

      <div className="mt-2 flex gap-4 text-xs text-gray-500">
        <span className="flex items-center gap-1.5">
          <span className="inline-block h-2 w-4 rounded-full bg-blue-500" />
          Outgoing
        </span>
        <span className="flex items-center gap-1.5">
          <span className="inline-block h-2 w-4 rounded-full bg-orange-400" />
          Incoming
        </span>
      </div>
    </div>
  )
}
