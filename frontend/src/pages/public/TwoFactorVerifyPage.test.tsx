import { describe, it, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '@/test/utils'
import { TwoFactorVerifyPage } from './TwoFactorVerifyPage'
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

describe('TwoFactorVerifyPage', () => {
  it('shows no pending verification when there is no pending challenge', () => {
    render(<TwoFactorVerifyPage />)

    expect(screen.getByText('No pending verification')).toBeInTheDocument()
    expect(screen.getByText('Please log in first.')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /go to login/i })).toBeInTheDocument()
  })

  it('shows 2FA form when pending challenge exists', () => {
    // TODO: as never cast needed because pendingChallenge type isn't exported from the store — export the type
    useAuthStore.setState({
      pendingChallenge: { challenge_token: 'test-challenge-token' },
    } as never)

    render(<TwoFactorVerifyPage />)

    expect(screen.getByText('Two-Factor Authentication')).toBeInTheDocument()
    expect(screen.getByLabelText('Authentication Code')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /verify/i })).toBeInTheDocument()
  })

  it('shows authenticator app description by default', () => {
    // TODO: as never cast needed because pendingChallenge type isn't exported from the store — export the type
    useAuthStore.setState({
      pendingChallenge: { challenge_token: 'test-challenge-token' },
    } as never)

    render(<TwoFactorVerifyPage />)

    expect(screen.getByText(/6-digit code from your authenticator app/i)).toBeInTheDocument()
  })

  it('switches to recovery code mode', async () => {
    const user = userEvent.setup()
    // TODO: as never cast needed because pendingChallenge type isn't exported from the store — export the type
    useAuthStore.setState({
      pendingChallenge: { challenge_token: 'test-challenge-token' },
    } as never)

    render(<TwoFactorVerifyPage />)

    await user.click(screen.getByText(/use a recovery code instead/i))

    expect(screen.getByText(/enter one of your recovery codes/i)).toBeInTheDocument()
    expect(screen.getByLabelText('Recovery Code')).toBeInTheDocument()
    expect(screen.getByText(/use authenticator app instead/i)).toBeInTheDocument()
  })

  it('shows back to login link', () => {
    // TODO: as never cast needed because pendingChallenge type isn't exported from the store — export the type
    useAuthStore.setState({
      pendingChallenge: { challenge_token: 'test-challenge-token' },
    } as never)

    render(<TwoFactorVerifyPage />)

    expect(screen.getByText(/back to login/i)).toBeInTheDocument()
  })
})
