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
} from '@/types'

export const authApi = {
  login: (data: LoginRequest): Promise<AuthResponse> =>
    apiClient.post('/v1/auth/login', data),

  register: (data: RegisterRequest): Promise<AuthResponse> =>
    apiClient.post('/v1/auth/register', data),

  logout: (): Promise<void> => apiClient.post('/v1/auth/logout'),

  refresh: (): Promise<AuthResponse> => apiClient.post('/v1/auth/refresh'),

  me: (): Promise<User> => apiClient.get('/v1/users/me'),

  requestMagicLink: (data: MagicLinkRequest): Promise<{ message: string }> =>
    apiClient.post('/v1/auth/magic-link', data),

  verifyMagicLink: (data: MagicLinkVerifyRequest): Promise<AuthResponse> =>
    apiClient.post('/v1/auth/magic-link/verify', data),

  requestPasswordReset: (data: PasswordResetRequest): Promise<{ message: string }> =>
    apiClient.post('/v1/auth/password-reset', data),

  confirmPasswordReset: (data: PasswordResetConfirmRequest): Promise<{ message: string }> =>
    apiClient.post('/v1/auth/password-reset/confirm', data),
}
