import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { RoleBadge } from '@/components/RoleBadge'
import { listUsers, createUser, deleteUser } from '@/api/users'
import type { Role } from '@/types/user'
import { ApiError } from '@/api/client'

const ROLES: Role[] = ['Owner', 'Admin', 'Operator', 'Viewer']

export function Users() {
  const qc = useQueryClient()
  const [form, setForm] = useState({ email: '', name: '', password: '', role: 'Viewer' as Role })
  const [formError, setFormError] = useState<string | null>(null)

  const { data } = useQuery({ queryKey: ['users'], queryFn: listUsers })

  const createMut = useMutation({
    mutationFn: createUser,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['users'] })
      setForm({ email: '', name: '', password: '', role: 'Viewer' })
      setFormError(null)
    },
    onError: (e) => setFormError(e instanceof ApiError ? e.message : 'Failed to create user'),
  })

  const deleteMut = useMutation({
    mutationFn: deleteUser,
    onSuccess: () => void qc.invalidateQueries({ queryKey: ['users'] }),
  })

  const submit = (e: React.FormEvent) => {
    e.preventDefault()
    createMut.mutate(form)
  }

  const users = data?.items ?? []

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-bold text-slate-900">Users</h1>

      <div className="rounded-xl border border-gray-200 bg-white shadow-sm">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-100 text-left text-gray-500">
              <th className="px-6 py-3 font-medium">Name</th>
              <th className="px-6 py-3 font-medium">Email</th>
              <th className="px-6 py-3 font-medium">Role</th>
              <th className="px-6 py-3 font-medium" />
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-50">
            {users.map((u) => (
              <tr key={u.id}>
                <td className="px-6 py-3 font-medium text-slate-900">{u.name}</td>
                <td className="px-6 py-3 text-gray-600">{u.email}</td>
                <td className="px-6 py-3"><RoleBadge role={u.role} /></td>
                <td className="px-6 py-3 text-right">
                  <button
                    onClick={() => deleteMut.mutate(u.id)}
                    className="text-xs font-medium text-red-500 hover:underline"
                  >
                    Remove
                  </button>
                </td>
              </tr>
            ))}
            {users.length === 0 && (
              <tr>
                <td colSpan={4} className="px-6 py-8 text-center text-gray-400">No users</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
        <h2 className="mb-4 font-semibold text-slate-900">Add User</h2>
        <form onSubmit={submit} className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Email</label>
            <input
              type="email" required value={form.email}
              onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
              className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Name</label>
            <input
              type="text" required value={form.name}
              onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
              className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Password</label>
            <input
              type="password" required value={form.password}
              onChange={(e) => setForm((f) => ({ ...f, password: e.target.value }))}
              className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700">Role</label>
            <select
              value={form.role}
              onChange={(e) => setForm((f) => ({ ...f, role: e.target.value as Role }))}
              className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            >
              {ROLES.map((r) => <option key={r} value={r}>{r}</option>)}
            </select>
          </div>
          {formError && (
            <p className="col-span-2 rounded-md bg-red-50 px-3 py-2 text-sm text-red-600">{formError}</p>
          )}
          <div className="col-span-2">
            <button
              type="submit" disabled={createMut.isPending}
              className="rounded-lg bg-blue-600 px-5 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:opacity-50"
            >
              {createMut.isPending ? 'Adding…' : 'Add User'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
