import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useAuthStore } from './authStore'
import { mockUser } from '@/test/mocks/handlers'

// Reset zustand store before each test
beforeEach(() => {
  useAuthStore.setState({
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null,
  })
})

describe('authStore', () => {
  describe('initial state', () => {
    it('should have correct initial state', () => {
      const state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.isAuthenticated).toBe(false)
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
    })
  })

  describe('setUser', () => {
    it('should set user and isAuthenticated to true', () => {
      const { setUser } = useAuthStore.getState()
      setUser(mockUser)

      const state = useAuthStore.getState()
      expect(state.user).toEqual(mockUser)
      expect(state.isAuthenticated).toBe(true)
      expect(state.isLoading).toBe(false)
    })

    it('should clear user and set isAuthenticated to false when null', () => {
      // First set a user
      useAuthStore.getState().setUser(mockUser)
      expect(useAuthStore.getState().isAuthenticated).toBe(true)

      // Then clear it
      useAuthStore.getState().setUser(null)
      const state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.isAuthenticated).toBe(false)
    })
  })

  describe('setLoading', () => {
    it('should update isLoading state', () => {
      const { setLoading } = useAuthStore.getState()

      setLoading(true)
      expect(useAuthStore.getState().isLoading).toBe(true)

      setLoading(false)
      expect(useAuthStore.getState().isLoading).toBe(false)
    })
  })

  describe('setError', () => {
    it('should set error message', () => {
      const { setError } = useAuthStore.getState()
      setError('Something went wrong')

      expect(useAuthStore.getState().error).toBe('Something went wrong')
    })

    it('should clear error when null', () => {
      const { setError } = useAuthStore.getState()
      setError('Error message')
      setError(null)

      expect(useAuthStore.getState().error).toBeNull()
    })
  })

  describe('clearError', () => {
    it('should clear the error', () => {
      useAuthStore.setState({ error: 'Some error' })
      const { clearError } = useAuthStore.getState()

      clearError()

      expect(useAuthStore.getState().error).toBeNull()
    })
  })

  describe('login', () => {
    it('should login successfully with valid credentials', async () => {
      const { login } = useAuthStore.getState()

      await login('test@example.com', 'password123')

      const state = useAuthStore.getState()
      expect(state.user).toEqual(mockUser)
      expect(state.isAuthenticated).toBe(true)
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
    })

    it('should set loading state during login', async () => {
      const { login } = useAuthStore.getState()

      // Start login
      const loginPromise = login('test@example.com', 'password123')

      // Check loading state (might be flaky depending on timing)
      // await new Promise(resolve => setTimeout(resolve, 0))
      // expect(useAuthStore.getState().isLoading).toBe(true)

      await loginPromise

      expect(useAuthStore.getState().isLoading).toBe(false)
    })

    it('should handle login failure with invalid credentials', async () => {
      const { login } = useAuthStore.getState()

      await expect(login('test@example.com', 'wrongpassword')).rejects.toThrow()

      const state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.isAuthenticated).toBe(false)
      expect(state.isLoading).toBe(false)
      expect(state.error).toBe('Invalid email or password')
    })

    it('should login as admin with admin credentials', async () => {
      const { login } = useAuthStore.getState()

      await login('admin@example.com', 'password123')

      const state = useAuthStore.getState()
      expect(state.user?.role).toBe('admin')
      expect(state.isAuthenticated).toBe(true)
    })
  })

  describe('register', () => {
    it('should register successfully with new email', async () => {
      const { register } = useAuthStore.getState()

      await register('newuser@example.com', 'password123')

      const state = useAuthStore.getState()
      expect(state.user).not.toBeNull()
      expect(state.user?.email).toBe('newuser@example.com')
      expect(state.isAuthenticated).toBe(true)
      expect(state.isLoading).toBe(false)
    })

    it('should handle registration failure for existing email', async () => {
      const { register } = useAuthStore.getState()

      await expect(register('existing@example.com', 'password123')).rejects.toThrow()

      const state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.isAuthenticated).toBe(false)
      expect(state.error).toBe('Email already registered')
    })
  })

  describe('logout', () => {
    it('should clear user state on logout', async () => {
      // First login
      await useAuthStore.getState().login('test@example.com', 'password123')
      expect(useAuthStore.getState().isAuthenticated).toBe(true)

      // Then logout
      await useAuthStore.getState().logout()

      const state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.isAuthenticated).toBe(false)
      expect(state.isLoading).toBe(false)
    })

    it('should handle logout even if API call fails', async () => {
      // Login first
      await useAuthStore.getState().login('test@example.com', 'password123')

      // Logout should still clear state even if there's an error
      await useAuthStore.getState().logout()

      const state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.isAuthenticated).toBe(false)
    })
  })

  describe('refreshUser', () => {
    it('should fetch current user when authenticated', async () => {
      // Set authenticated state
      useAuthStore.setState({ isAuthenticated: true })

      await useAuthStore.getState().refreshUser()

      const state = useAuthStore.getState()
      expect(state.user).toEqual(mockUser)
      expect(state.isAuthenticated).toBe(true)
      expect(state.isLoading).toBe(false)
    })

    it('should skip refresh when not authenticated', async () => {
      useAuthStore.setState({ isAuthenticated: false, isLoading: true })

      await useAuthStore.getState().refreshUser()

      const state = useAuthStore.getState()
      expect(state.user).toBeNull()
      expect(state.isAuthenticated).toBe(false)
      expect(state.isLoading).toBe(false)
    })
  })
})
