import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '@/test/utils'
import { SettingsPage } from './SettingsPage'
import { useAuthStore } from '@/stores/authStore'
import { mockUser } from '@/test/mocks/handlers'

beforeEach(() => {
  useAuthStore.setState({
    user: mockUser,
    isAuthenticated: true,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
})

describe('SettingsPage', () => {
  it('renders settings page heading', () => {
    render(<SettingsPage />)

    expect(screen.getByText('Settings')).toBeInTheDocument()
    expect(screen.getByText('Manage your account settings and preferences.')).toBeInTheDocument()
  })

  it('shows account information section', async () => {
    render(<SettingsPage />)

    expect(screen.getByText('Account Information')).toBeInTheDocument()
    expect(screen.getByText(mockUser.email)).toBeInTheDocument()
  })

  it('shows email verified status', () => {
    render(<SettingsPage />)

    expect(screen.getByText('Verified')).toBeInTheDocument()
  })

  it('shows membership status', () => {
    render(<SettingsPage />)

    expect(screen.getByText('active')).toBeInTheDocument()
  })

  it('shows change email form', () => {
    render(<SettingsPage />)

    // Use heading role to avoid matching parent container with same text content
    expect(screen.getByRole('heading', { name: 'Change Email' })).toBeInTheDocument()
    expect(screen.getByLabelText('New Email Address')).toBeInTheDocument()
  })

  it('shows change password form', () => {
    render(<SettingsPage />)

    expect(screen.getByRole('heading', { name: 'Change Password' })).toBeInTheDocument()
    // Two "Current Password" fields exist (email form + password form); get the password form one
    const currentPasswordInputs = screen.getAllByLabelText('Current Password')
    expect(currentPasswordInputs.length).toBeGreaterThan(0)
    expect(screen.getByLabelText('New Password')).toBeInTheDocument()
    expect(screen.getByLabelText('Confirm New Password')).toBeInTheDocument()
  })

  it('shows 2FA section', async () => {
    render(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByText('Two-Factor Authentication')).toBeInTheDocument()
    })
  })

  it('shows enable 2FA button when 2FA is disabled', async () => {
    render(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByText('Enable Two-Factor Authentication')).toBeInTheDocument()
    })
  })

  it('shows success message after password change', async () => {
    const user = userEvent.setup()
    render(<SettingsPage />)

    // Two "Current Password" fields exist; the password form's field is the last one
    const currentPasswordInputs = screen.getAllByLabelText('Current Password')
    await user.type(currentPasswordInputs[currentPasswordInputs.length - 1], 'OldPassword123!')
    await user.type(screen.getByLabelText('New Password'), 'NewPassword123!')
    await user.type(screen.getByLabelText('Confirm New Password'), 'NewPassword123!')
    await user.click(screen.getByRole('button', { name: /update password/i }))

    await waitFor(() => {
      expect(screen.getByText('Password updated successfully!')).toBeInTheDocument()
    })
  })

  it('shows admin badge for admin users', () => {
    useAuthStore.setState({
      user: { ...mockUser, role: 'admin' as const },
      isAuthenticated: true,
    })

    render(<SettingsPage />)

    expect(screen.getByText('Admin')).toBeInTheDocument()
  })
})
