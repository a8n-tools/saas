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

// DEV-525: the per-user opt-out for the new-login-location alert.
describe('SettingsPage login-location alerts', () => {
  it('reflects the stored preference', () => {
    setupAuthUser({ ...mockUser, login_location_alerts: false })
    render(<SettingsPage />)

    expect(screen.getByLabelText('New sign-in location alerts')).not.toBeChecked()
  })

  it('defaults to on', () => {
    render(<SettingsPage />)

    expect(screen.getByLabelText('New sign-in location alerts')).toBeChecked()
  })

  it('persists a turn-off and updates the auth store', async () => {
    render(<SettingsPage />)

    await userEvent.click(screen.getByLabelText('New sign-in location alerts'))

    await waitFor(() => {
      expect(useAuthStore.getState().user?.login_location_alerts).toBe(false)
    })
  })
})

// DEV-525: profile picture upload / removal.
describe('SettingsPage avatar', () => {
  it('shows the initials fallback when no avatar is set', () => {
    render(<SettingsPage />)

    expect(screen.getByLabelText('No profile picture set')).toHaveTextContent('T')
    expect(screen.queryByAltText('Your profile picture')).not.toBeInTheDocument()
  })

  it('offers no Remove until an avatar exists', () => {
    render(<SettingsPage />)

    expect(screen.queryByRole('button', { name: /^remove$/i })).not.toBeInTheDocument()
  })

  it('uploads a selected file and records the new version', async () => {
    render(<SettingsPage />)

    const file = new File([new Uint8Array([0x89, 0x50, 0x4e, 0x47])], 'me.png', {
      type: 'image/png',
    })
    await userEvent.upload(screen.getByLabelText('Choose a profile picture'), file)

    await waitFor(() => {
      expect(useAuthStore.getState().user?.avatar_updated_at).toBe('2026-08-04T00:00:00Z')
    })
  })

  it('renders the stored image once one exists', async () => {
    setupAuthUser({ ...mockUser, avatar_updated_at: '2026-08-04T00:00:00Z' })
    render(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByAltText('Your profile picture')).toBeInTheDocument()
    })
    expect(screen.queryByLabelText('No profile picture set')).not.toBeInTheDocument()
  })

  it('removes an existing avatar', async () => {
    setupAuthUser({ ...mockUser, avatar_updated_at: '2026-08-04T00:00:00Z' })
    render(<SettingsPage />)

    await userEvent.click(screen.getByRole('button', { name: /^remove$/i }))

    await waitFor(() => {
      expect(useAuthStore.getState().user?.avatar_updated_at).toBeNull()
    })
  })
})
