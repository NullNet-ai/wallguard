import type { Role } from '@/types/user'

const styles: Record<Role, string> = {
  Owner:    'bg-purple-100 text-purple-700',
  Admin:    'bg-blue-100 text-blue-700',
  Operator: 'bg-teal-100 text-teal-700',
  Viewer:   'bg-gray-100 text-gray-600',
}

export function RoleBadge({ role }: { role: Role }) {
  return (
    <span className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${styles[role]}`}>
      {role}
    </span>
  )
}
