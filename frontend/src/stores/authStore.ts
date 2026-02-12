import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { User } from '@/types'
import { authApi } from '@/api'

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null

  // Actions
  setUser: (user: User | null) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  refreshUser: () => Promise<void>
  clearError: () => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: true,
      error: null,

      setUser: (user) =>
        set({
          user,
          isAuthenticated: !!user,
          isLoading: false,
        }),

      setLoading: (isLoading) => set({ isLoading }),

      setError: (error) => set({ error }),

      clearError: () => set({ error: null }),

      login: async (email, password) => {
        set({ isLoading: true, error: null })
        try {
          const response = await authApi.login({ email, password })
          set({
            user: response.user,
            isAuthenticated: true,
            isLoading: false,
          })
        } catch (err) {
          const error = err as { error?: { message?: string } }
          set({
            error: error.error?.message || 'Login failed',
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
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
          })
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
