import { describe, it, expect, beforeEach, vi } from 'vitest'

import { render, setupAuthUser } from '@/test/utils'
import { LogoutPage } from './LogoutPage'
import { useAuthStore } from '@/stores/authStore'

beforeEach(() => {
  setupAuthUser()
  Object.defineProperty(window, 'location', {
    configurable: true,
    value: { ...window.location, replace: vi.fn(), reload: vi.fn(), assign: vi.fn() },
  })
})

describe('LogoutPage', () => {
  it('shows loading spinner', () => {
    render(<LogoutPage />)

    // TODO: query by role="status" or aria-label instead of CSS class
    const spinner = document.querySelector('.animate-spin')
    expect(spinner).toBeInTheDocument()
  })

  it('calls logout on mount', async () => {
    const logout = vi.fn().mockResolvedValue(undefined)
    // TODO: as never cast needed because Zustand setState doesn't accept partial function overrides — fix store type
    useAuthStore.setState({ logout } as never)

    render(<LogoutPage />)

    await vi.waitFor(() => {
      expect(logout).toHaveBeenCalled()
    })
  })
})
