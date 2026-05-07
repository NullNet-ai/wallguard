import type { FailureSeverity } from '@/types/failure'

const styles: Record<FailureSeverity, string> = {
  Warning: 'bg-amber-100 text-amber-700',
  Error:   'bg-orange-100 text-orange-700',
  Fatal:   'bg-red-100 text-red-700',
}

export function SeverityBadge({ severity }: { severity: FailureSeverity }) {
  return (
    <span className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${styles[severity]}`}>
      {severity}
    </span>
  )
}
