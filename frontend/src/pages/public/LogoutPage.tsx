import { useEffect } from 'react'
import { useSearchParams, useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { Loader2 } from 'lucide-react'

/**
 * Handles SSO logout for child apps.
 * Child apps redirect here: /logout?redirect=<url>
 * This page clears auth state (cookies + localStorage) then redirects.
 */
export function LogoutPage() {
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const logout = useAuthStore((s) => s.logout)

  useEffect(() => {
    const redirect = searchParams.get('redirect')

    logout().then(() => {
      if (redirect && redirect.startsWith('http')) {
        window.location.href = redirect
      } else {
        navigate('/login', { replace: true })
      }
    })
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="flex items-center justify-center min-h-[calc(100vh-8rem)]">
      <Loader2 className="h-8 w-8 animate-spin text-primary" />
    </div>
  )
}
