import { useToastStore } from './use-toast'
import { cn } from '@/lib/utils'

export function Toaster() {
  const toasts = useToastStore()

  if (toasts.length === 0) return null

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2" aria-live="polite">
      {toasts.map((t) => (
        <div
          key={t.id}
          role="status"
          className={cn(
            'rounded-md border bg-background px-4 py-3 text-sm shadow-lg',
            'animate-in slide-in-from-bottom-2',
          )}
        >
          {t.message}
        </div>
      ))}
    </div>
  )
}
