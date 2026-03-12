import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render, setupAuthUser } from '@/test/utils'
import { DashboardPage } from './DashboardPage'
import { useAuthStore } from '@/stores/authStore'
import { mockUser } from '@/test/mocks/handlers'

beforeEach(() => {
  setupAuthUser()
})

describe('DashboardPage', () => {
  it('renders welcome message', () => {
    render(<DashboardPage />)

    expect(screen.getByText('Welcome back!')).toBeInTheDocument()
  })

  it('shows membership section', () => {
    render(<DashboardPage />)

    expect(screen.getByText('Membership')).toBeInTheDocument()
  })

  it('shows active membership status for active user', () => {
    render(<DashboardPage />)

    expect(screen.getByText('Active')).toBeInTheDocument()
    expect(screen.getByText(/you have access to all applications/i)).toBeInTheDocument()
  })

  it('shows subscribe button for user without active membership', () => {
    useAuthStore.setState({
      user: { ...mockUser, membership_status: 'none' as never },
      isAuthenticated: true,
    })

    render(<DashboardPage />)

    expect(screen.getByRole('button', { name: /subscribe now/i })).toBeInTheDocument()
  })

  it('shows applications section', async () => {
    render(<DashboardPage />)

    expect(screen.getByText('Your Applications')).toBeInTheDocument()

    await waitFor(() => {
      expect(screen.getByText('RUS')).toBeInTheDocument()
    })
  })

  it('shows manage button for active membership', () => {
    render(<DashboardPage />)

    expect(screen.getByRole('button', { name: /manage/i })).toBeInTheDocument()
  })
})
