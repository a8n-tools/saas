import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAuthUser } from '@/test/utils'
import { SettingsPage } from './SettingsPage'
import { useAuthStore } from '@/stores/authStore'
import { mockUser } from '@/test/mocks/handlers'

beforeEach(() => {
  setupAuthUser()
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

// DEV-525: the optional profile fields, which flow to relying parties as the
// OIDC `profile` and `phone` claims.
describe('SettingsPage profile form', () => {
  it('prefills from the signed-in user', () => {
    setupAuthUser({ ...mockUser, first_name: 'Ada', last_name: 'Lovelace', phone: '+61 400 000 000' })
    render(<SettingsPage />)

    expect(screen.getByLabelText('First name')).toHaveValue('Ada')
    expect(screen.getByLabelText('Last name')).toHaveValue('Lovelace')
    expect(screen.getByLabelText('Phone')).toHaveValue('+61 400 000 000')
  })

  it('renders empty fields when nothing is set', () => {
    render(<SettingsPage />)

    expect(screen.getByLabelText('First name')).toHaveValue('')
    expect(screen.getByLabelText('Phone')).toHaveValue('')
  })

  it('keeps save disabled until something changes', async () => {
    render(<SettingsPage />)

    const save = screen.getByRole('button', { name: /save profile/i })
    expect(save).toBeDisabled()

    await userEvent.type(screen.getByLabelText('First name'), 'Ada')
    await waitFor(() => expect(save).toBeEnabled())
  })

  it('saves trimmed values and puts them in the auth store', async () => {
    render(<SettingsPage />)

    await userEvent.type(screen.getByLabelText('First name'), '  Ada  ')
    await userEvent.click(screen.getByRole('button', { name: /save profile/i }))

    await waitFor(() => {
      expect(useAuthStore.getState().user?.first_name).toBe('Ada')
    })
    expect(screen.getByText('Profile saved.')).toBeInTheDocument()
  })

  it('clears a field rather than storing an empty string', async () => {
    setupAuthUser({ ...mockUser, first_name: 'Ada' })
    render(<SettingsPage />)

    await userEvent.clear(screen.getByLabelText('First name'))
    await userEvent.click(screen.getByRole('button', { name: /save profile/i }))

    // null, not '': "unset" has to stay one state, not two that look alike.
    await waitFor(() => {
      expect(useAuthStore.getState().user?.first_name).toBeNull()
    })
  })

  it('rejects a value past the 64-character ceiling without calling the API', async () => {
    render(<SettingsPage />)

    await userEvent.type(screen.getByLabelText('First name'), 'a'.repeat(65))
    await userEvent.click(screen.getByRole('button', { name: /save profile/i }))

    await waitFor(() => {
      expect(screen.getByText('Must be 64 characters or fewer')).toBeInTheDocument()
    })
    expect(useAuthStore.getState().user?.first_name).toBeNull()
  })
})
