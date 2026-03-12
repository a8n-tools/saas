import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '@/test/utils'
import { RegisterPage } from './RegisterPage'
import { useAuthStore } from '@/stores/authStore'

beforeEach(() => {
  useAuthStore.setState({
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
})

describe('RegisterPage', () => {
  it('renders registration form', () => {
    render(<RegisterPage />)

    expect(screen.getByText('Create an account')).toBeInTheDocument()
    expect(screen.getByLabelText('Email')).toBeInTheDocument()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
    expect(screen.getByLabelText('Confirm Password')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /create account/i })).toBeInTheDocument()
  })

  it('shows password requirements checklist', () => {
    render(<RegisterPage />)

    expect(screen.getByText('At least 12 characters')).toBeInTheDocument()
    expect(screen.getByText('One lowercase letter')).toBeInTheDocument()
    expect(screen.getByText('One uppercase letter')).toBeInTheDocument()
    expect(screen.getByText('One number')).toBeInTheDocument()
    expect(screen.getByText('One special character')).toBeInTheDocument()
  })

  it('shows links to login and magic link', () => {
    render(<RegisterPage />)

    expect(screen.getByText(/sign in/i)).toBeInTheDocument()
    expect(screen.getByText(/sign up with magic link/i)).toBeInTheDocument()
    expect(screen.getByText(/terms of service/i)).toBeInTheDocument()
    expect(screen.getByText(/privacy policy/i)).toBeInTheDocument()
  })

  it('shows validation error for invalid email', async () => {
    const user = userEvent.setup()
    render(<RegisterPage />)

    // Submit with empty email to trigger zod validation without native form constraint
    await user.click(screen.getByRole('button', { name: /create account/i }))

    await waitFor(() => {
      expect(screen.getByText('Invalid email address')).toBeInTheDocument()
    })
  })

  it('shows error for mismatched passwords', async () => {
    const user = userEvent.setup()
    render(<RegisterPage />)

    await user.type(screen.getByLabelText('Email'), 'new@example.com')
    await user.type(screen.getByLabelText('Password'), 'ValidPass123!')
    await user.type(screen.getByLabelText('Confirm Password'), 'DifferentPass123!')
    await user.click(screen.getByRole('button', { name: /create account/i }))

    await waitFor(() => {
      expect(screen.getByText("Passwords don't match")).toBeInTheDocument()
    })
  })

  it('shows success state after successful registration', async () => {
    const user = userEvent.setup()
    render(<RegisterPage />)

    await user.type(screen.getByLabelText('Email'), 'newuser@example.com')
    await user.type(screen.getByLabelText('Password'), 'ValidPass123!')
    await user.type(screen.getByLabelText('Confirm Password'), 'ValidPass123!')
    await user.click(screen.getByRole('button', { name: /create account/i }))

    await waitFor(() => {
      expect(screen.getByText('Account created!')).toBeInTheDocument()
    })
  })

  it('shows error when email is already registered', async () => {
    const user = userEvent.setup()
    render(<RegisterPage />)

    await user.type(screen.getByLabelText('Email'), 'existing@example.com')
    await user.type(screen.getByLabelText('Password'), 'ValidPass123!')
    await user.type(screen.getByLabelText('Confirm Password'), 'ValidPass123!')
    await user.click(screen.getByRole('button', { name: /create account/i }))

    await waitFor(() => {
      expect(screen.getByText('Email already registered')).toBeInTheDocument()
    })
  })
})
