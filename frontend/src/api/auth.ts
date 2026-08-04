import { apiClient } from './client'
import { config } from '@/config'
import type {
  User,
  AuthResponse,
  LoginRequest,
  RegisterRequest,
  MagicLinkRequest,
  MagicLinkVerifyRequest,
  PasswordResetRequest,
  PasswordResetConfirmRequest,
  TwoFactorChallengeResponse,
  TwoFactorSetupResponse,
  RecoveryCodesResponse,
  TwoFactorStatusResponse,
} from '@/types'

export const authApi = {
  login: (data: LoginRequest): Promise<AuthResponse | TwoFactorChallengeResponse> =>
    apiClient.post('/auth/login', data),

  register: (data: RegisterRequest): Promise<AuthResponse> =>
    apiClient.post('/auth/register', data),

  logout: (): Promise<void> => apiClient.post('/auth/logout'),

  refresh: (): Promise<AuthResponse> => apiClient.post('/auth/refresh'),

  me: (): Promise<User> => apiClient.get('/users/me'),

  requestMagicLink: (data: MagicLinkRequest): Promise<{ message: string }> =>
    apiClient.post('/auth/magic-link', data),

  verifyMagicLink: (data: MagicLinkVerifyRequest): Promise<AuthResponse | TwoFactorChallengeResponse> =>
    apiClient.post('/auth/magic-link/verify', data),

  requestPasswordReset: (data: PasswordResetRequest): Promise<{ message: string }> =>
    apiClient.post('/auth/password-reset', data),

  confirmPasswordReset: (data: PasswordResetConfirmRequest): Promise<{ message: string }> =>
    apiClient.post('/auth/password-reset/confirm', data),

  changePassword: (data: { current_password: string; new_password: string }): Promise<void> =>
    apiClient.put('/users/me/password', data),

  // DEV-525. A full replace, not a patch: every field is written on every call,
  // so omitting one clears it. Returns the refreshed user for the auth store.
  updateProfile: (data: {
    first_name?: string | null
    last_name?: string | null
    phone?: string | null
  }): Promise<User> => apiClient.put('/users/me/profile', data),

  // DEV-525: per-user opt-out for the new-login-location alert.
  updateLoginAlerts: (data: { enabled: boolean }): Promise<User> =>
    apiClient.put('/users/me/login-alerts', data),

  // DEV-525: avatar upload / removal. Both return the refreshed user, so the
  // caller gets the new `avatar_updated_at` without a second request.
  uploadAvatar: (file: File): Promise<User> => {
    const form = new FormData()
    form.append('avatar', file)
    return apiClient.post('/users/me/avatar', form)
  },

  deleteAvatar: (): Promise<User> => apiClient.delete('/users/me/avatar'),

  /**
   * DEV-525: fetch the stored avatar as an object URL.
   *
   * Deliberately not routed through `apiClient`, which rejects any non-JSON
   * response. Fetched as a blob rather than pointed at with `<img src>` so the
   * auth cookie is sent explicitly with `credentials: 'include'`, instead of
   * depending on the browser attaching it to a cross-origin image request.
   *
   * Returns null when the user has no avatar (404), which is the signal to
   * render the initials fallback. The caller owns the object URL and must
   * revoke it.
   */
  fetchAvatarObjectUrl: async (): Promise<string | null> => {
    const res = await fetch(`${config.apiUrl}/v1/users/me/avatar`, {
      credentials: 'include',
    })
    if (!res.ok) return null
    return URL.createObjectURL(await res.blob())
  },

  requestEmailChange: (data: { new_email: string; current_password?: string }): Promise<{ message: string; requires_relogin: boolean }> =>
    apiClient.post('/users/me/email', data),

  confirmEmailChange: (data: { token: string }): Promise<{ message: string }> =>
    apiClient.post('/users/me/email/confirm', data),

  requestEmailVerification: (): Promise<{ message: string }> =>
    apiClient.post('/users/me/email/verify'),

  confirmEmailVerification: (data: { token: string }): Promise<{ message: string; subscription_tier: string }> =>
    apiClient.post('/users/me/email/verify/confirm', data),

  // 2FA endpoints
  setup2FA: (): Promise<TwoFactorSetupResponse> =>
    apiClient.post('/auth/2fa/setup'),

  confirm2FA: (data: { code: string }): Promise<RecoveryCodesResponse> =>
    apiClient.post('/auth/2fa/confirm', data),

  verify2FA: (data: { challenge_token: string; code: string }): Promise<AuthResponse> =>
    apiClient.post('/auth/2fa/verify', data),

  disable2FA: (data: { password: string }): Promise<void> =>
    apiClient.post('/auth/2fa/disable', data),

  regenerateRecoveryCodes: (data: { password: string }): Promise<RecoveryCodesResponse> =>
    apiClient.post('/auth/2fa/recovery-codes', data),

  get2FAStatus: (): Promise<TwoFactorStatusResponse> =>
    apiClient.get('/auth/2fa/status'),

  acceptInvite: (data: { token: string; password?: string }): Promise<AuthResponse | { needs_password: true; email: string }> =>
    apiClient.post('/auth/invite/accept', data),

  setupStatus: (): Promise<{ setup_required: boolean; email_enabled: boolean; stripe_enabled: boolean }> =>
    apiClient.get('/auth/setup/status'),

  setup: (data: { email: string; password: string }): Promise<AuthResponse> =>
    apiClient.post('/auth/setup', data),

  deleteAccount: (data: { password: string; totp_code?: string }): Promise<void> =>
    apiClient.delete('/users/me', data),
}
