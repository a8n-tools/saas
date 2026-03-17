import { describe, it, expect, vi, afterEach } from 'vitest'
import { cn, formatDate, formatCurrency, formatRelativeTime } from './utils'

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
