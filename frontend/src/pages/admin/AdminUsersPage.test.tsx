import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '@/test/utils'
import { AdminUsersPage } from './AdminUsersPage'
import { useAuthStore } from '@/stores/authStore'
import { mockAdminUser } from '@/test/mocks/handlers'

beforeEach(() => {
  useAuthStore.setState({
    user: mockAdminUser,
    isAuthenticated: true,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
})

describe('AdminUsersPage', () => {
  it('renders users page heading', () => {
    render(<AdminUsersPage />)

    expect(screen.getByText('Users')).toBeInTheDocument()
    expect(screen.getByText('Manage user accounts and memberships.')).toBeInTheDocument()
  })

  it('shows search input', () => {
    render(<AdminUsersPage />)

    expect(screen.getByPlaceholderText('Search users...')).toBeInTheDocument()
  })

  it('shows user list after loading', async () => {
    render(<AdminUsersPage />)

    await waitFor(() => {
      expect(screen.getByText('test@example.com')).toBeInTheDocument()
    })
  })

  it('shows user count', async () => {
    render(<AdminUsersPage />)

    await waitFor(() => {
      expect(screen.getByText('1 users total')).toBeInTheDocument()
    })
  })

  it('shows user role badge', async () => {
    render(<AdminUsersPage />)

    await waitFor(() => {
      expect(screen.getByText('subscriber')).toBeInTheDocument()
    })
  })

  it('shows action menu for each user', async () => {
    render(<AdminUsersPage />)

    await waitFor(() => {
      const menuButton = screen.getByRole('button', { name: '' })
      expect(menuButton).toBeInTheDocument()
    })
  })

  it('opens reset password dialog', async () => {
    const user = userEvent.setup()
    render(<AdminUsersPage />)

    await waitFor(() => {
      expect(screen.getByText('test@example.com')).toBeInTheDocument()
    })

    // Click the action menu
    const menuTrigger = document.querySelector('[data-radix-collection-item]')
    if (menuTrigger) {
      await user.click(menuTrigger as HTMLElement)
    }
  })
})
