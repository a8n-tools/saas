import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupUnauthUser } from '@/test/utils'
import { SetupPage } from './SetupPage'

beforeEach(() => {
  setupUnauthUser()
})

describe('SetupPage', () => {
  it('renders the setup form', () => {
    render(<SetupPage />)

    expect(screen.getByText('Initial Setup')).toBeInTheDocument()
    expect(screen.getByText(/create the first admin account/i)).toBeInTheDocument()
    expect(screen.getByLabelText('Admin Email')).toBeInTheDocument()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
    expect(screen.getByLabelText('Confirm Password')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /create admin account/i })).toBeInTheDocument()
  })

  it('shows password requirement indicators', () => {
    render(<SetupPage />)

    expect(screen.getByText('At least 12 characters')).toBeInTheDocument()
    expect(screen.getByText('One lowercase letter')).toBeInTheDocument()
    expect(screen.getByText('One uppercase letter')).toBeInTheDocument()
    expect(screen.getByText('One number')).toBeInTheDocument()
    expect(screen.getByText('One special character')).toBeInTheDocument()
  })

  it('shows validation errors for empty form submit', async () => {
    const user = userEvent.setup()
    render(<SetupPage />)

    await user.click(screen.getByRole('button', { name: /create admin account/i }))

    await waitFor(() => {
      expect(screen.getByText('Invalid email address')).toBeInTheDocument()
    })
  })

  it('shows validation error for short password', async () => {
    const user = userEvent.setup()
    render(<SetupPage />)

    await user.type(screen.getByLabelText('Admin Email'), 'admin@example.com')
    await user.type(screen.getByLabelText('Password'), 'short')
    await user.type(screen.getByLabelText('Confirm Password'), 'short')
    await user.click(screen.getByRole('button', { name: /create admin account/i }))

    await waitFor(() => {
      expect(screen.getByText(/at least 12 characters/i)).toBeInTheDocument()
    })
  })

  it('shows success state after successful setup', async () => {
    const user = userEvent.setup()
    render(<SetupPage />)

    await user.type(screen.getByLabelText('Admin Email'), 'newadmin@example.com')
    await user.type(screen.getByLabelText('Password'), 'SecurePass123!')
    await user.type(screen.getByLabelText('Confirm Password'), 'SecurePass123!')
    await user.click(screen.getByRole('button', { name: /create admin account/i }))

    await waitFor(() => {
      expect(screen.getByText('Admin account created!')).toBeInTheDocument()
    })
    expect(screen.getByText(/redirecting to dashboard/i)).toBeInTheDocument()
  })

  it('shows error when setup fails', async () => {
    const user = userEvent.setup()
    render(<SetupPage />)

    await user.type(screen.getByLabelText('Admin Email'), 'taken@example.com')
    await user.type(screen.getByLabelText('Password'), 'SecurePass123!')
    await user.type(screen.getByLabelText('Confirm Password'), 'SecurePass123!')
    await user.click(screen.getByRole('button', { name: /create admin account/i }))

    await waitFor(() => {
      expect(screen.getByText('Setup already completed')).toBeInTheDocument()
    })
  })
})
