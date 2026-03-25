import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { hasActiveMembership } from '@/lib/utils'

export function useAuth() {
  const store = useAuthStore()
  const navigate = useNavigate()

  useEffect(() => {
    // Refresh user on mount if authenticated
    if (store.isAuthenticated) {
      store.refreshUser()
    } else {
      store.setLoading(false)
    }
  }, [])

  const loginAndRedirect = async (email: string, password: string) => {
    await store.login(email, password)
    navigate('/dashboard')
  }

  const registerAndRedirect = async (email: string, password: string) => {
    await store.register(email, password)
    navigate('/dashboard')
  }

  const logoutAndRedirect = async () => {
    await store.logout()
    navigate('/')
  }

  return {
    user: store.user,
    isAuthenticated: store.isAuthenticated,
    isLoading: store.isLoading,
    error: store.error,
    login: loginAndRedirect,
    register: registerAndRedirect,
    logout: logoutAndRedirect,
    clearError: store.clearError,
    hasActiveMembership: hasActiveMembership(store.user),
    isAdmin: store.user?.role === 'admin',
  }
}
