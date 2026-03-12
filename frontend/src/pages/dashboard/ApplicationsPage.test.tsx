import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render, setupAuthUser } from '@/test/utils'
import { ApplicationsPage } from './ApplicationsPage'
import { useAuthStore } from '@/stores/authStore'
import { mockUser } from '@/test/mocks/handlers'

beforeEach(() => {
  setupAuthUser()
})

describe('ApplicationsPage', () => {
  it('renders page heading', async () => {
    render(<ApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByText('Applications')).toBeInTheDocument()
    })
  })

  it('shows applications list after loading', async () => {
    render(<ApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByText('RUS')).toBeInTheDocument()
    })
  })

  it('shows Launch button for accessible app', async () => {
    render(<ApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /launch/i })).toBeInTheDocument()
    })
  })

  it('shows membership required banner for user without active membership', async () => {
    useAuthStore.setState({
      user: { ...mockUser, membership_status: 'none' as never },
      isAuthenticated: true,
    })

    render(<ApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByText('Membership required')).toBeInTheDocument()
    })
  })

  it('shows subscribe button when membership required', async () => {
    useAuthStore.setState({
      user: { ...mockUser, membership_status: 'none' as never },
      isAuthenticated: true,
    })

    render(<ApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /subscribe now/i })).toBeInTheDocument()
    })
  })
})
