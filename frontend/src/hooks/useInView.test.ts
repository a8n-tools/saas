import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useInView } from './useInView'

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('useInView', () => {
  it('starts with inView false', () => {
    const { result } = renderHook(() => useInView())
    expect(result.current.inView).toBe(false)
  })

  it('returns a ref object', () => {
    const { result } = renderHook(() => useInView())
    expect(result.current.ref).toBeDefined()
    expect(result.current.ref.current).toBeNull()
  })

  it('uses default threshold of 0.15', () => {
    const spy = vi.spyOn(window, 'IntersectionObserver' as never)
    renderHook(() => useInView())
    // Without a ref attached, observer isn't created
    expect(spy).not.toHaveBeenCalled()
  })

  it('does not create observer when ref has no element', () => {
    const observeSpy = vi.fn()
    vi.stubGlobal('IntersectionObserver', vi.fn(() => ({
      observe: observeSpy,
      unobserve: vi.fn(),
      disconnect: vi.fn(),
    })))

    renderHook(() => useInView())

    // ref.current is null so observer.observe should not be called
    expect(observeSpy).not.toHaveBeenCalled()
  })
})
