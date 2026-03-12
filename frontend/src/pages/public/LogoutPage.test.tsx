import { describe, it, expect, beforeEach, vi } from 'vitest'

import { render } from '@/test/utils'
import { LogoutPage } from './LogoutPage'
import { useAuthStore } from '@/stores/authStore'
import { mockUser } from '@/test/mocks/handlers'

beforeEach(() => {
  useAuthStore.setState({
    user: mockUser,
    isAuthenticated: true,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
  Object.defineProperty(window, 'location', {
    configurable: true,
    value: { ...window.location, replace: vi.fn(), reload: vi.fn(), assign: vi.fn() },
  })
})

describe('LogoutPage', () => {
  it('shows loading spinner', () => {
    render(<LogoutPage />)

    // The spinner is rendered during the logout process
    const spinner = document.querySelector('.animate-spin')
    expect(spinner).toBeInTheDocument()
  })

  it('calls logout on mount', async () => {
    const logout = vi.fn().mockResolvedValue(undefined)
    useAuthStore.setState({ logout } as never)

    render(<LogoutPage />)

    await vi.waitFor(() => {
      expect(logout).toHaveBeenCalled()
    })
  })
})
