import { describe, it, expect } from 'vitest'
import { authApi } from './auth'
import { mockUser } from '@/test/mocks/handlers'

describe('authApi', () => {
  describe('login', () => {
    it('should return user and token on successful login', async () => {
      const response = await authApi.login({
        email: 'test@example.com',
        password: 'password123',
      })

      expect(response.user).toEqual(mockUser)
      expect(response.access_token).toBe('mock-access-token')
    })

    it('should throw error on invalid credentials', async () => {
      await expect(
        authApi.login({
          email: 'test@example.com',
          password: 'wrongpassword',
        })
      ).rejects.toMatchObject({
        error: {
          code: 'INVALID_CREDENTIALS',
          message: 'Invalid email or password',
        },
      })
    })
  })

  describe('register', () => {
    it('should return user and token on successful registration', async () => {
      const response = await authApi.register({
        email: 'newuser@example.com',
        password: 'password123',
      })

      expect(response.user.email).toBe('newuser@example.com')
      expect(response.access_token).toBe('mock-access-token')
    })

    it('should throw error when email already exists', async () => {
      await expect(
        authApi.register({
          email: 'existing@example.com',
          password: 'password123',
        })
      ).rejects.toMatchObject({
        error: {
          code: 'EMAIL_EXISTS',
          message: 'Email already registered',
        },
      })
    })
  })

  describe('logout', () => {
    it('should complete logout successfully', async () => {
      await expect(authApi.logout()).resolves.not.toThrow()
    })
  })

  describe('refresh', () => {
    it('should return refreshed user and token', async () => {
      const response = await authApi.refresh()

      expect(response.user).toEqual(mockUser)
      expect(response.access_token).toBe('mock-refreshed-token')
    })
  })

  describe('me', () => {
    it('should return current user', async () => {
      const user = await authApi.me()

      expect(user).toEqual(mockUser)
    })
  })

  describe('requestMagicLink', () => {
    it('should send magic link request successfully', async () => {
      const response = await authApi.requestMagicLink({
        email: 'test@example.com',
      })

      expect(response.message).toBe('Magic link sent to your email')
    })
  })

  describe('verifyMagicLink', () => {
    it('should verify valid magic link token', async () => {
      const response = await authApi.verifyMagicLink({
        token: 'valid-token',
      })

      expect(response.user).toEqual(mockUser)
      expect(response.access_token).toBe('mock-access-token')
    })

    it('should throw error on invalid token', async () => {
      await expect(
        authApi.verifyMagicLink({
          token: 'invalid-token',
        })
      ).rejects.toMatchObject({
        error: {
          code: 'INVALID_TOKEN',
          message: 'Invalid or expired token',
        },
      })
    })
  })

  describe('requestPasswordReset', () => {
    it('should send password reset request successfully', async () => {
      const response = await authApi.requestPasswordReset({
        email: 'test@example.com',
      })

      expect(response.message).toContain('reset link has been sent')
    })
  })

  describe('confirmPasswordReset', () => {
    it('should reset password with valid token', async () => {
      const response = await authApi.confirmPasswordReset({
        token: 'valid-reset-token',
        password: 'newpassword123',
      })

      expect(response.message).toContain('Password has been reset')
    })

    it('should throw error on invalid reset token', async () => {
      await expect(
        authApi.confirmPasswordReset({
          token: 'invalid-token',
          password: 'newpassword123',
        })
      ).rejects.toMatchObject({
        error: {
          code: 'INVALID_TOKEN',
          message: 'Invalid or expired token',
        },
      })
    })
  })
})
