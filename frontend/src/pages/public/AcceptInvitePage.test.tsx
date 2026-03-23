import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupUnauthUser } from '@/test/utils'
import { AcceptInvitePage } from './AcceptInvitePage'

beforeEach(() => {
  setupUnauthUser()
})

// Helper to set query params before rendering
function renderWithToken(token?: string) {
  const search = token ? `?token=${token}` : ''
  window.history.pushState({}, '', `/accept-invite${search}`)
  return render(<AcceptInvitePage />)
}

describe('AcceptInvitePage', () => {
  it('shows error when no token is provided', async () => {
    renderWithToken()

    await waitFor(() => {
      expect(screen.getByText('Invitation Failed')).toBeInTheDocument()
    })
    expect(screen.getByText('No invite token provided')).toBeInTheDocument()
    expect(screen.getByText('Go to Login')).toBeInTheDocument()
  })

  it('shows error for invalid token', async () => {
    renderWithToken('bad-token')

    await waitFor(() => {
      expect(screen.getByText('Invitation Failed')).toBeInTheDocument()
    })
    expect(screen.getByText('Invalid or expired invite link')).toBeInTheDocument()
  })

  it('shows password form when invite needs password', async () => {
    renderWithToken('valid-invite-token')

    await waitFor(() => {
      expect(screen.getByText('Create Your Account')).toBeInTheDocument()
    })
    expect(screen.getByText(/invited@example.com/)).toBeInTheDocument()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
    expect(screen.getByLabelText('Confirm Password')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /create account/i })).toBeInTheDocument()
  })

  it('shows loading state initially with token', () => {
    renderWithToken('valid-invite-token')

    expect(screen.getByText('Verifying invite...')).toBeInTheDocument()
  })

  it('validates password requirements on submit', async () => {
    const user = userEvent.setup()
    renderWithToken('valid-invite-token')

    await waitFor(() => {
      expect(screen.getByText('Create Your Account')).toBeInTheDocument()
    })

    await user.type(screen.getByLabelText('Password'), 'short')
    await user.type(screen.getByLabelText('Confirm Password'), 'short')
    await user.click(screen.getByRole('button', { name: /create account/i }))

    await waitFor(() => {
      expect(screen.getByText(/at least 12 characters/i)).toBeInTheDocument()
    })
  })
})
