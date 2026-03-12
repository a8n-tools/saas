import { describe, it, expect } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '@/test/utils'
import { PasswordResetPage } from './PasswordResetPage'

describe('PasswordResetPage', () => {
  it('renders password reset form', () => {
    render(<PasswordResetPage />)

    expect(screen.getByText('Reset your password')).toBeInTheDocument()
    expect(screen.getByLabelText('Email')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /send reset link/i })).toBeInTheDocument()
    expect(screen.getByText(/remember your password/i)).toBeInTheDocument()
  })

  it('shows validation error for invalid email', async () => {
    const user = userEvent.setup()
    render(<PasswordResetPage />)

    // Submit with empty email to trigger zod validation without native form constraint
    await user.click(screen.getByRole('button', { name: /send reset link/i }))

    await waitFor(() => {
      expect(screen.getByText('Invalid email address')).toBeInTheDocument()
    })
  })

  it('shows success state after submitting email', async () => {
    const user = userEvent.setup()
    render(<PasswordResetPage />)

    await user.type(screen.getByLabelText('Email'), 'test@example.com')
    await user.click(screen.getByRole('button', { name: /send reset link/i }))

    await waitFor(() => {
      expect(screen.getByText('Check your email')).toBeInTheDocument()
      expect(screen.getByText(/reset instructions/i)).toBeInTheDocument()
    })
  })

  it('shows back to login link in success state', async () => {
    const user = userEvent.setup()
    render(<PasswordResetPage />)

    await user.type(screen.getByLabelText('Email'), 'test@example.com')
    await user.click(screen.getByRole('button', { name: /send reset link/i }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /back to login/i })).toBeInTheDocument()
    })
  })
})
