import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid,
  Tooltip, ResponsiveContainer,
} from 'recharts'
import { getDeviceMetrics } from '@/api/metrics'
import type { MetricsPoint } from '@/types/metrics'

// ---------------------------------------------------------------------------
// Presets — metrics arrive every 30 s so Live uses a 30 s bucket
// ---------------------------------------------------------------------------

const PRESETS = [
  { label: 'Live', window: '10m', bucket: '30s', live: true,  refresh: 30_000 },
  { label: '1h',   window: '1h',  bucket: '1m',  live: false, refresh: 60_000 },
  { label: '6h',   window: '6h',  bucket: '5m',  live: false, refresh: 60_000 },
  { label: '24h',  window: '24h', bucket: '15m', live: false, refresh: 60_000 },
  { label: '7d',   window: '7d',  bucket: '1h',  live: false, refresh: 60_000 },
]

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function fmtBytes(b: number): string {
  if (b >= 1_073_741_824) return `${(b / 1_073_741_824).toFixed(1)} GB`
  if (b >= 1_048_576)     return `${(b / 1_048_576).toFixed(1)} MB`
  if (b >= 1_024)         return `${(b / 1_024).toFixed(1)} KB`
  return `${b} B`
}

function fmtTime(ts: number, window: string): string {
  const d = new Date(ts)
  if (window === '7d')  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
  if (window === '24h') return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
  return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', second: window === '10m' ? '2-digit' : undefined })
}

// ---------------------------------------------------------------------------
// Stat card
// ---------------------------------------------------------------------------

function StatCard({ label, value, sub }: { label: string; value: string; sub?: string }) {
  return (
    <div className="rounded-lg border border-gray-200 bg-gray-50 px-4 py-3">
      <p className="text-xs text-gray-500">{label}</p>
      <p className="mt-0.5 text-xl font-semibold text-slate-900">{value}</p>
      {sub && <p className="text-xs text-gray-400">{sub}</p>}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Small area chart
// ---------------------------------------------------------------------------

function MetricChart({
  data, dataKey, label, color, domain, tickFormatter, window: win,
}: {
  data:          object[]
  dataKey:       string
  label:         string
  color:         string
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  domain:        any[]
  tickFormatter: (v: number) => string
  window:        string
}) {
  return (
    <div>
      <p className="mb-1 text-xs font-medium text-gray-500">{label}</p>
      <ResponsiveContainer width="100%" height={120}>
        <AreaChart data={data} margin={{ top: 2, right: 4, left: 0, bottom: 0 }}>
          <defs>
            <linearGradient id={`grad-${dataKey}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%"  stopColor={color} stopOpacity={0.2} />
              <stop offset="95%" stopColor={color} stopOpacity={0.01} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
          <XAxis
            dataKey="t"
            type="number"
            domain={['dataMin', 'dataMax']}
            scale="time"
            tickFormatter={(v) => fmtTime(v as number, win)}
            tick={{ fontSize: 10, fill: '#94a3b8' }}
            tickLine={false}
            axisLine={false}
            minTickGap={50}
          />
          <YAxis
            domain={domain}
            tickFormatter={tickFormatter}
            tick={{ fontSize: 10, fill: '#94a3b8' }}
            tickLine={false}
            axisLine={false}
            width={48}
          />
          <Tooltip
            formatter={(v) => [tickFormatter(v as number), label]}
            labelFormatter={(ts) => new Date(ts as number).toLocaleString()}
            contentStyle={{ fontSize: 12, borderRadius: 8, border: '1px solid #e2e8f0' }}
          />
          <Area
            type="monotone"
            dataKey={dataKey}
            stroke={color}
            strokeWidth={1.5}
            fill={`url(#grad-${dataKey})`}
            dot={false}
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function TelemetryDashboard({ deviceId }: { deviceId: string }) {
  const [preset, setPreset] = useState(PRESETS[0])

  const { data, isLoading } = useQuery({
    queryKey: ['metrics', deviceId, preset.window, preset.bucket],
    queryFn: () => getDeviceMetrics(deviceId, preset.window, preset.bucket),
    refetchInterval: preset.refresh,
  })

  const latest: MetricsPoint | undefined = data?.points[data.points.length - 1]

  const chartData = useMemo(
    () =>
      (data?.points ?? []).map((p) => ({
        t:        p.t,
        cpu:      parseFloat(p.cpu_percent.toFixed(1)),
        mem_pct:  p.mem_total_bytes > 0
          ? parseFloat(((p.mem_used_bytes / p.mem_total_bytes) * 100).toFixed(1))
          : 0,
        disk_pct: p.disk_total_bytes > 0
          ? parseFloat(((p.disk_used_bytes / p.disk_total_bytes) * 100).toFixed(1))
          : 0,
        load:     parseFloat(p.load_1m.toFixed(2)),
      })),
    [data],
  )

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <h2 className="font-semibold text-slate-900">Telemetry</h2>
        <div className="flex gap-1">
          {PRESETS.map((p) => (
            <button
              key={p.label}
              onClick={() => setPreset(p)}
              className={`flex items-center gap-1.5 rounded px-3 py-1 text-xs font-medium transition-colors ${
                preset.label === p.label
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-500 hover:bg-gray-100'
              }`}
            >
              {p.live && (
                <span className={`inline-block h-1.5 w-1.5 rounded-full ${
                  preset.label === p.label ? 'bg-white animate-pulse' : 'bg-red-500'
                }`} />
              )}
              {p.label}
            </button>
          ))}
        </div>
      </div>

      {/* Stat cards */}
      <div className="mb-4 grid grid-cols-4 gap-3">
        <StatCard
          label="CPU"
          value={latest ? `${latest.cpu_percent.toFixed(1)}%` : '—'}
        />
        <StatCard
          label="Memory"
          value={latest && latest.mem_total_bytes > 0
            ? `${((latest.mem_used_bytes / latest.mem_total_bytes) * 100).toFixed(1)}%`
            : '—'}
          sub={latest && latest.mem_total_bytes > 0
            ? `${fmtBytes(latest.mem_used_bytes)} / ${fmtBytes(latest.mem_total_bytes)}`
            : undefined}
        />
        <StatCard
          label="Disk"
          value={latest && latest.disk_total_bytes > 0
            ? `${((latest.disk_used_bytes / latest.disk_total_bytes) * 100).toFixed(1)}%`
            : '—'}
          sub={latest && latest.disk_total_bytes > 0
            ? `${fmtBytes(latest.disk_used_bytes)} / ${fmtBytes(latest.disk_total_bytes)}`
            : undefined}
        />
        <StatCard
          label="Load (1m)"
          value={latest ? latest.load_1m.toFixed(2) : '—'}
        />
      </div>

      {/* Charts */}
      {isLoading ? (
        <div className="flex h-40 items-center justify-center text-sm text-gray-400">Loading…</div>
      ) : chartData.length === 0 ? (
        <div className="flex h-40 items-center justify-center text-sm text-gray-400">
          No telemetry data in this window
        </div>
      ) : (
        <div className="grid grid-cols-2 gap-6">
          <MetricChart
            data={chartData}
            dataKey="cpu"
            label="CPU %"
            color="#2563eb"
            domain={[0, 100]}
            tickFormatter={(v) => `${v}%`}
            window={preset.window}
          />
          <MetricChart
            data={chartData}
            dataKey="mem_pct"
            label="Memory %"
            color="#7c3aed"
            domain={[0, 100]}
            tickFormatter={(v) => `${v}%`}
            window={preset.window}
          />
          <MetricChart
            data={chartData}
            dataKey="disk_pct"
            label="Disk %"
            color="#0891b2"
            domain={[0, 100]}
            tickFormatter={(v) => `${v}%`}
            window={preset.window}
          />
          <MetricChart
            data={chartData}
            dataKey="load"
            label="Load avg (1m)"
            color="#d97706"
            domain={[0, 'auto'] as [number, 'auto']}
            tickFormatter={(v) => (v as number).toFixed(1)}
            window={preset.window}
          />
        </div>
      )}
    </div>
  )
}
