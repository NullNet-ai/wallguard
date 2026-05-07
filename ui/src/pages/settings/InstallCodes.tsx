import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { listInstallCodes, createInstallCode } from '@/api/install_codes'

function formatExpiry(expiresAt: string | null): string {
  if (!expiresAt) return 'No expiry'
  const diff = new Date(expiresAt).getTime() - Date.now()
  if (diff < 0) return 'Expired'
  const mins  = Math.floor(diff / 60_000)
  const hours = Math.floor(mins / 60)
  const days  = Math.floor(hours / 24)
  if (days > 0)  return `Expires in ${days}d`
  if (hours > 0) return `Expires in ${hours}h`
  return `Expires in ${mins}m`
}

export function InstallCodes() {
  const qc = useQueryClient()
  const [ttl, setTtl]         = useState('24')
  const [newCode, setNewCode] = useState<string | null>(null)
  const [copied, setCopied]   = useState(false)

  const { data } = useQuery({ queryKey: ['install-codes'], queryFn: listInstallCodes })

  const createMut = useMutation({
    mutationFn: () => createInstallCode(ttl ? Number(ttl) : undefined),
    onSuccess: (code) => {
      setNewCode(code.code)
      void qc.invalidateQueries({ queryKey: ['install-codes'] })
    },
  })

  const copy = () => {
    if (!newCode) return
    void navigator.clipboard.writeText(newCode)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const codes = data?.items ?? []

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-bold text-slate-900">Installation Codes</h1>

      <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
        <h2 className="mb-4 font-semibold text-slate-900">Generate Code</h2>
        <div className="flex items-end gap-4">
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">TTL (hours)</label>
            <input
              type="number" min="1" max="720" value={ttl}
              onChange={(e) => setTtl(e.target.value)}
              className="w-32 rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
          <button
            onClick={() => createMut.mutate()}
            disabled={createMut.isPending}
            className="rounded-lg bg-blue-600 px-5 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:opacity-50"
          >
            {createMut.isPending ? 'Generating…' : 'Generate'}
          </button>
        </div>

        {newCode && (
          <div className="mt-4 flex items-center gap-3 rounded-lg bg-slate-900 px-5 py-3">
            <code className="flex-1 font-mono text-sm tracking-widest text-green-400">{newCode}</code>
            <button
              onClick={copy}
              className="rounded-md bg-white/10 px-3 py-1 text-xs font-medium text-white hover:bg-white/20"
            >
              {copied ? 'Copied!' : 'Copy'}
            </button>
          </div>
        )}
      </div>

      <div className="rounded-xl border border-gray-200 bg-white shadow-sm">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-100 text-left text-gray-500">
              <th className="px-6 py-3 font-medium">Code</th>
              <th className="px-6 py-3 font-medium">Status</th>
              <th className="px-6 py-3 font-medium">Expiry</th>
              <th className="px-6 py-3 font-medium">Created</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-50">
            {codes.map((c) => (
              <tr key={c.code} className={c.used ? 'opacity-50' : ''}>
                <td className="px-6 py-3 font-mono font-medium text-slate-900">{c.code}</td>
                <td className="px-6 py-3">
                  <span className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${c.used ? 'bg-gray-100 text-gray-500' : 'bg-green-100 text-green-700'}`}>
                    {c.used ? 'Used' : 'Active'}
                  </span>
                </td>
                <td className="px-6 py-3 text-gray-500">{formatExpiry(c.expires_at)}</td>
                <td className="px-6 py-3 text-gray-500">{new Date(c.created_at).toLocaleDateString()}</td>
              </tr>
            ))}
            {codes.length === 0 && (
              <tr>
                <td colSpan={4} className="px-6 py-8 text-center text-gray-400">No codes generated</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  )
}
