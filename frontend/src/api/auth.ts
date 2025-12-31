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
    apiClient.post('/auth/login', data),

  register: (data: RegisterRequest): Promise<AuthResponse> =>
    apiClient.post('/auth/register', data),

  logout: (): Promise<void> => apiClient.post('/auth/logout'),

  refresh: (): Promise<AuthResponse> => apiClient.post('/auth/refresh'),

  me: (): Promise<User> => apiClient.get('/users/me'),

  requestMagicLink: (data: MagicLinkRequest): Promise<{ message: string }> =>
    apiClient.post('/auth/magic-link', data),

  verifyMagicLink: (data: MagicLinkVerifyRequest): Promise<AuthResponse> =>
    apiClient.post('/auth/magic-link/verify', data),

  requestPasswordReset: (data: PasswordResetRequest): Promise<{ message: string }> =>
    apiClient.post('/auth/password-reset', data),

  confirmPasswordReset: (data: PasswordResetConfirmRequest): Promise<{ message: string }> =>
    apiClient.post('/auth/password-reset/confirm', data),
}
