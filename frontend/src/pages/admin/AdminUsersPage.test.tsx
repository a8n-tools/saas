import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAdminUser } from '@/test/utils'
import { AdminUsersPage } from './AdminUsersPage'

beforeEach(() => {
  setupAdminUser()
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
      expect(screen.getByRole('button', { name: /open user actions/i })).toBeInTheDocument()
    })
  })

  it('opens reset password dialog', async () => {
    const user = userEvent.setup()
    render(<AdminUsersPage />)

    await waitFor(() => {
      expect(screen.getByText('test@example.com')).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /open user actions/i }))
  })
})
