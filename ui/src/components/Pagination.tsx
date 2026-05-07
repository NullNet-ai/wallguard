interface Props {
  offset: number
  limit: number
  total: number
  onPrev: () => void
  onNext: () => void
}

export function Pagination({ offset, limit, total, onPrev, onNext }: Props) {
  const from = total === 0 ? 0 : offset + 1
  const to   = Math.min(offset + limit, total)

  return (
    <div className="flex items-center justify-between border-t border-gray-200 pt-4">
      <p className="text-sm text-gray-500">
        {from}–{to} of {total}
      </p>
      <div className="flex gap-2">
        <button
          onClick={onPrev}
          disabled={offset === 0}
          className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-40"
        >
          Previous
        </button>
        <button
          onClick={onNext}
          disabled={to >= total}
          className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-40"
        >
          Next
        </button>
      </div>
    </div>
  )
}
