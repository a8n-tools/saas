import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { User, TwoFactorChallengeResponse } from '@/types'
import { authApi } from '@/api'

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null
  pendingChallenge: { challenge_token: string } | null

  // Actions
  setUser: (user: User | null) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  refreshUser: () => Promise<void>
  clearError: () => void
  verify2FA: (code: string) => Promise<void>
  clearPendingChallenge: () => void
}

function isTwoFactorChallenge(response: unknown): response is TwoFactorChallengeResponse {
  return (
    typeof response === 'object' &&
    response !== null &&
    'requires_2fa' in response &&
    (response as TwoFactorChallengeResponse).requires_2fa === true
  )
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: true,
      error: null,
      pendingChallenge: null,

      setUser: (user) =>
        set({
          user,
          isAuthenticated: !!user,
          isLoading: false,
        }),

      setLoading: (isLoading) => set({ isLoading }),

      setError: (error) => set({ error }),

      clearError: () => set({ error: null }),

      clearPendingChallenge: () => set({ pendingChallenge: null }),

      login: async (email, password) => {
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.login({ email, password })
          if (isTwoFactorChallenge(response)) {
            set({
              pendingChallenge: { challenge_token: response.challenge_token },
              isLoading: false,
            })
          } else {
            set({
              user: response.user,
              isAuthenticated: true,
              isLoading: false,
              pendingChallenge: null,
            })
          }
        } catch (err) {
          const error = err as { error?: { message?: string } }
          set({
            error: error.error?.message || 'Login failed',
            isLoading: false,
          })
          throw err
        }
      },

      verify2FA: async (code: string) => {
        const { pendingChallenge } = get()
        if (!pendingChallenge) {
          throw new Error('No pending 2FA challenge')
        }
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.verify2FA({
            challenge_token: pendingChallenge.challenge_token,
            code,
          })
          set({
            user: response.user,
            isAuthenticated: true,
            isLoading: false,
            pendingChallenge: null,
          })
        } catch (err) {
          const error = err as { error?: { message?: string } }
          set({
            error: error.error?.message || 'Verification failed',
            isLoading: false,
          })
          throw err
        }
      },

      register: async (email, password) => {
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.register({ email, password })
          set({
            user: response.user,
            isAuthenticated: true,
            isLoading: false,
          })
        } catch (err) {
          const error = err as { error?: { message?: string } }
          set({
            error: error.error?.message || 'Registration failed',
            isLoading: false,
          })
          throw err
        }
      },

      logout: async () => {
        try {
          await authApi.logout()
        } catch {
          // Ignore logout errors
        } finally {
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            pendingChallenge: null,
          })
        }
      },

      refreshUser: async () => {
        const { isAuthenticated } = get()
        if (!isAuthenticated) {
          set({ isLoading: false })
          return
        }

        // Don't set isLoading here — this is a background refresh.
        // Setting isLoading causes ProtectedRoute to unmount/remount children,
        // which re-triggers mount effects and creates an infinite loop.
        try {
          const user = await authApi.me()
          set({
            user,
            isAuthenticated: true,
            isLoading: false,
          })
        } catch {
          // Access token may be expired — try refreshing it
          try {
            // Refresh sets a new JWT cookie but doesn't return user data
            await authApi.refresh()
            // Now fetch user with the fresh token
            const user = await authApi.me()
            set({
              user,
              isAuthenticated: true,
              isLoading: false,
            })
          } catch {
            // Refresh token also failed — truly logged out
            set({
              user: null,
              isAuthenticated: false,
              isLoading: false,
            })
          }
        }
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)

// Sync auth state across tabs — when another tab/window changes localStorage
// (e.g., login or logout), update this tab's in-memory state.
if (typeof window !== 'undefined') {
  window.addEventListener('storage', (e) => {
    if (e.key === 'auth-storage') {
      if (!e.newValue) {
        // Key was removed — logged out
        useAuthStore.setState({
          user: null,
          isAuthenticated: false,
          isLoading: false,
          pendingChallenge: null,
        })
      } else {
        try {
          const parsed = JSON.parse(e.newValue)
          const isAuthenticated = parsed?.state?.isAuthenticated ?? false
          if (!isAuthenticated) {
            useAuthStore.setState({
              user: null,
              isAuthenticated: false,
              isLoading: false,
              pendingChallenge: null,
            })
          } else if (isAuthenticated && !useAuthStore.getState().isAuthenticated) {
            // Another tab logged in — mark authenticated and refresh user data
            useAuthStore.setState({ isAuthenticated: true, isLoading: false })
            useAuthStore.getState().refreshUser()
          }
        } catch {
          // Ignore parse errors
        }
      }
    }
  })
}
