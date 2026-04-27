// Minimal toast store — no Radix dependency needed for state management.
// The Toaster component subscribes to this store and renders toasts.
import { useState, useEffect } from 'react'

export interface Toast {
  id: string
  message: string
}

type Listener = (toasts: Toast[]) => void

let toasts: Toast[] = []
const listeners = new Set<Listener>()

function notify() {
  listeners.forEach((l) => l([...toasts]))
}

let counter = 0
export function toast(message: string) {
  const id = String(++counter)
  toasts = [...toasts, { id, message }]
  notify()
  setTimeout(() => {
    toasts = toasts.filter((t) => t.id !== id)
    notify()
  }, 4000)
}

export function useToastStore(): Toast[] {
  const [state, setState] = useState<Toast[]>([...toasts])
  useEffect(() => {
    listeners.add(setState)
    return () => { listeners.delete(setState) }
  }, [])
  return state
}
