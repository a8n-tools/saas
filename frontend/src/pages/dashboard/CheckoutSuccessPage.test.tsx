import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render } from '@/test/utils'
import { CheckoutSuccessPage } from './CheckoutSuccessPage'
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
})

describe('CheckoutSuccessPage', () => {
  it('renders success message', () => {
    render(<CheckoutSuccessPage />)

    expect(screen.getByText(/welcome aboard/i)).toBeInTheDocument()
    expect(screen.getByText(/membership is now active/i)).toBeInTheDocument()
  })

  it('shows membership tier info for logged in user', async () => {
    render(<CheckoutSuccessPage />)

    await waitFor(() => {
      expect(screen.getByText(/personal plan/i)).toBeInTheDocument()
    })
  })

  it('shows navigation buttons', () => {
    render(<CheckoutSuccessPage />)

    expect(screen.getByRole('button', { name: /browse applications/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /view membership details/i })).toBeInTheDocument()
  })

  it('shows countdown timer', async () => {
    render(<CheckoutSuccessPage />)

    await waitFor(() => {
      expect(screen.getByText(/redirecting to applications in/i)).toBeInTheDocument()
    })
  })
})
