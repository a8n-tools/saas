import { describe, it, expect, vi, afterEach } from 'vitest'
import { cn, formatDate, formatCurrency, formatRelativeTime, hasActiveMembership } from './utils'
import type { User } from '@/types'

describe('cn', () => {
  it('merges class names', () => {
    expect(cn('px-2', 'py-1')).toBe('px-2 py-1')
  })

  it('handles conditional classes', () => {
    expect(cn('px-2', false && 'hidden', 'py-1')).toBe('px-2 py-1')
  })

  it('resolves tailwind conflicts', () => {
    expect(cn('px-2', 'px-4')).toBe('px-4')
  })
})

describe('formatDate', () => {
  it('formats a date string', () => {
    const result = formatDate('2024-06-15T00:00:00Z')
    expect(result).toBe('June 15, 2024')
  })

  it('formats a Date object', () => {
    const result = formatDate(new Date('2024-01-01T00:00:00Z'))
    expect(result).toBe('January 1, 2024')
  })
})

describe('formatCurrency', () => {
  it('formats cents to dollars', () => {
    expect(formatCurrency(300)).toBe('$3.00')
  })

  it('formats zero cents', () => {
    expect(formatCurrency(0)).toBe('$0.00')
  })

  it('supports other currencies', () => {
    const result = formatCurrency(1500, 'eur')
    expect(result).toContain('15.00')
  })
})

describe('formatRelativeTime', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('returns "just now" for less than 1 minute ago', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-06-15T12:00:30Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('just now')
  })

  it('returns minutes ago', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-06-15T12:10:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('10 minutes ago')
  })

  it('returns hours ago', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-06-15T15:00:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('3 hours ago')
  })

  it('returns "1 hour ago"', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-06-15T13:00:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('1 hour ago')
  })

  it('returns "yesterday"', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-06-16T12:00:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('yesterday')
  })

  it('returns days ago', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-06-20T12:00:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('5 days ago')
  })

  it('returns weeks ago', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-07-06T12:00:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('3 weeks ago')
  })

  it('returns months ago', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2024-09-15T12:00:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('3 months ago')
  })

  it('returns years ago', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-06-15T12:00:00Z'))
    expect(formatRelativeTime('2024-06-15T12:00:00Z')).toBe('2 years ago')
  })
})

describe('hasActiveMembership', () => {
  const baseUser: User = {
    id: '1',
    email: 'test@example.com',
    role: 'subscriber',
    email_verified: true,
    two_factor_enabled: false,
    membership_status: 'none',
    membership_tier: null,
    price_locked: false,
    locked_price_id: null,
    locked_price_amount: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    subscription_tier: 'trial_1m',
    trial_ends_at: null,
    lifetime_member: false,
  }

  it('returns false for null user', () => {
    expect(hasActiveMembership(null)).toBe(false)
  })

  it('returns false for undefined user', () => {
    expect(hasActiveMembership(undefined)).toBe(false)
  })

  it('returns false for non-member', () => {
    expect(hasActiveMembership(baseUser)).toBe(false)
  })

  it('returns true for admin', () => {
    expect(hasActiveMembership({ ...baseUser, role: 'admin' })).toBe(true)
  })

  it('returns true for active subscription', () => {
    expect(hasActiveMembership({ ...baseUser, membership_status: 'active' })).toBe(true)
  })

  it('returns true for grace period', () => {
    expect(hasActiveMembership({ ...baseUser, membership_status: 'grace_period' })).toBe(true)
  })

  it('returns true for lifetime member', () => {
    expect(hasActiveMembership({ ...baseUser, lifetime_member: true })).toBe(true)
  })

  it('returns true for active trial', () => {
    const future = new Date(Date.now() + 86400000).toISOString()
    expect(hasActiveMembership({ ...baseUser, trial_ends_at: future })).toBe(true)
  })

  it('returns false for expired trial', () => {
    const past = new Date(Date.now() - 86400000).toISOString()
    expect(hasActiveMembership({ ...baseUser, trial_ends_at: past })).toBe(false)
  })

  it('returns false for past_due', () => {
    expect(hasActiveMembership({ ...baseUser, membership_status: 'past_due' })).toBe(false)
  })

  it('returns false for canceled', () => {
    expect(hasActiveMembership({ ...baseUser, membership_status: 'canceled' })).toBe(false)
  })
})
