import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'
import type { User } from '@/types'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatDate(date: string | Date): string {
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(new Date(date))
}

export function formatCurrency(cents: number, currency = 'usd'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency.toUpperCase(),
  }).format(cents / 100)
}

export const APP_GRADIENTS = [
  'from-indigo-500 to-primary',
  'from-teal-500 to-indigo-500',
  'from-primary to-teal-500',
  'from-violet-500 to-indigo-500',
]

export const getAppGradient = (index: number) => APP_GRADIENTS[index % APP_GRADIENTS.length]

/** Check if a user has active membership access. Mirrors backend AccessTokenClaims::has_member_access(). */
export function hasActiveMembership(user: User | null | undefined): boolean {
  if (!user) return false
  return (
    user.role === 'admin' ||
    user.lifetime_member ||
    (user.trial_ends_at != null && new Date(user.trial_ends_at) > new Date()) ||
    user.membership_status === 'active' ||
    user.membership_status === 'grace_period'
  )
}

export function formatRelativeTime(date: string | Date): string {
  const now = new Date()
  const then = new Date(date)
  const diffMs = now.getTime() - then.getTime()
  const isFuture = diffMs < 0
  const absDiffMs = Math.abs(diffMs)
  const diffDays = Math.floor(absDiffMs / (1000 * 60 * 60 * 24))
  const suffix = isFuture ? '' : ' ago'
  const prefix = isFuture ? 'in ' : ''

  if (diffDays === 0) {
    const diffHours = Math.floor(absDiffMs / (1000 * 60 * 60))
    if (diffHours === 0) {
      const diffMinutes = Math.floor(absDiffMs / (1000 * 60))
      if (diffMinutes <= 1) return isFuture ? 'in a moment' : 'just now'
      return `${prefix}${diffMinutes} minutes${suffix}`
    }
    return `${prefix}${diffHours} hour${diffHours === 1 ? '' : 's'}${suffix}`
  }

  if (diffDays === 1) return isFuture ? 'in 1 day' : 'yesterday'
  if (diffDays < 7) return `${prefix}${diffDays} days${suffix}`
  if (diffDays < 30) return `${prefix}${Math.floor(diffDays / 7)} week${Math.floor(diffDays / 7) === 1 ? '' : 's'}${suffix}`
  if (diffDays < 365) return `${prefix}${Math.floor(diffDays / 30)} month${Math.floor(diffDays / 30) === 1 ? '' : 's'}${suffix}`
  return `${prefix}${Math.floor(diffDays / 365)} year${Math.floor(diffDays / 365) === 1 ? '' : 's'}${suffix}`
}
