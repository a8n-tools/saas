import { apiClient } from './client'
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

  setupStatus: (): Promise<{ setup_required: boolean }> =>
    apiClient.get('/auth/setup/status'),

  setup: (data: { email: string; password: string }): Promise<AuthResponse> =>
    apiClient.post('/auth/setup', data),

  deleteAccount: (data: { password: string; totp_code?: string }): Promise<void> =>
    apiClient.delete('/users/me', data),
}
