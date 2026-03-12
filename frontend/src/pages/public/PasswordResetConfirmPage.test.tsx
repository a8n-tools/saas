import { describe, it, expect } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { render as rtlRender } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PasswordResetConfirmPage } from './PasswordResetConfirmPage'

function renderWithToken(search = '') {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return rtlRender(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[`/password-reset/confirm${search}`]}>
        <Routes>
          <Route path="/password-reset/confirm" element={<PasswordResetConfirmPage />} />
          <Route path="/login" element={<div>Login Page</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('PasswordResetConfirmPage', () => {
  it('shows invalid link state when no token', () => {
    renderWithToken()

    expect(screen.getByText('Invalid Reset Link')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /request new reset link/i })).toBeInTheDocument()
  })

  it('shows password form when token is present', () => {
    renderWithToken('?token=some-token')

    expect(screen.getByText('Set new password')).toBeInTheDocument()
    expect(screen.getByLabelText('New Password')).toBeInTheDocument()
    expect(screen.getByLabelText('Confirm Password')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /reset password/i })).toBeInTheDocument()
  })

  it('shows password requirements checklist', () => {
    renderWithToken('?token=some-token')

    expect(screen.getByText('At least 12 characters')).toBeInTheDocument()
    expect(screen.getByText('One lowercase letter')).toBeInTheDocument()
    expect(screen.getByText('One uppercase letter')).toBeInTheDocument()
  })

  it('shows validation error for mismatched passwords', async () => {
    const user = userEvent.setup()
    renderWithToken('?token=some-token')

    await user.type(screen.getByLabelText('New Password'), 'ValidPass123!')
    await user.type(screen.getByLabelText('Confirm Password'), 'DifferentPass123!')
    await user.click(screen.getByRole('button', { name: /reset password/i }))

    await waitFor(() => {
      expect(screen.getByText("Passwords don't match")).toBeInTheDocument()
    })
  })

  it('navigates to login with success param after successful reset', async () => {
    const user = userEvent.setup()
    renderWithToken('?token=valid-reset-token')

    await user.type(screen.getByLabelText('New Password'), 'ValidPass123!')
    await user.type(screen.getByLabelText('Confirm Password'), 'ValidPass123!')
    await user.click(screen.getByRole('button', { name: /reset password/i }))

    await waitFor(() => {
      expect(screen.getByText('Login Page')).toBeInTheDocument()
    })
  })
})
