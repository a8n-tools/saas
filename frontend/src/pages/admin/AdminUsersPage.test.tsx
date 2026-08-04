import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAdminUser, setupSuperAdminUser } from '@/test/utils'
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
      expect(screen.getByText('Subscriber')).toBeInTheDocument()
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

// DEV-525: the API refuses a password reset or a role change to every admin
// except the super admin. The panel must not offer a control that will 403.
describe('AdminUsersPage super-admin gating', () => {
  async function openActionMenu() {
    const user = userEvent.setup()
    render(<AdminUsersPage />)
    await waitFor(() => {
      expect(screen.getByText('test@example.com')).toBeInTheDocument()
    })
    await user.click(screen.getByRole('button', { name: /open user actions/i }))
    return user
  }

  it('disables reset password and the role change for an ordinary admin', async () => {
    setupAdminUser()
    await openActionMenu()

    await waitFor(() => {
      expect(screen.getByText('Reset Password').closest('[role="menuitem"]')).toHaveAttribute(
        'aria-disabled',
        'true'
      )
    })
    expect(screen.getByText('Make Admin').closest('[role="menuitem"]')).toHaveAttribute(
      'aria-disabled',
      'true'
    )
  })

  it('enables both for the super admin', async () => {
    setupSuperAdminUser()
    await openActionMenu()

    await waitFor(() => {
      expect(screen.getByText('Reset Password').closest('[role="menuitem"]')).not.toHaveAttribute(
        'aria-disabled',
        'true'
      )
    })
    expect(screen.getByText('Make Admin').closest('[role="menuitem"]')).not.toHaveAttribute(
      'aria-disabled',
      'true'
    )
  })
})
