import { useState, useCallback, useEffect, useRef } from 'react'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useAuthStore } from '@/stores/authStore'
import { config } from '@/config'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2 } from 'lucide-react'

const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(1, 'Password is required'),
  remember: z.boolean().optional(),
})

type LoginFormData = z.infer<typeof loginSchema>

/**
 * Get the redirect URL from query params.
 * Returns the raw redirect param (validated server-side), or '/dashboard' as default.
 */
function getRedirectUrl(params: URLSearchParams): string {
  const redirect = params.get('redirect')
  if (!redirect) return '/dashboard'
  // Relative paths are used directly
  if (redirect.startsWith('/') && !redirect.startsWith('//')) {
    return redirect
  }
  // External URLs are validated server-side via /v1/auth/redirect
  return redirect
}

export function LoginPage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const { login, error, clearError } = useAuthStore()
  const [isLoading, setIsLoading] = useState(false)
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  })

  const isAuthenticated = useAuthStore((s) => s.isAuthenticated)
  const redirectUrl = getRedirectUrl(searchParams)
  const isExternal = redirectUrl.startsWith('http')
  // "checked" flag means the API already verified we're not logged in — show the form
  const alreadyChecked = searchParams.has('checked')

  // For external redirects, go through the server-side /v1/auth/redirect endpoint
  // which refreshes the access_token cookie if the refresh token is still valid.
  // The "checked" flag from the server is the loop breaker — if the server can't
  // authenticate us, it redirects back with ?checked=1 so we show the login form.
  const shouldServerRedirect = isExternal && !alreadyChecked

  const doRedirect = useCallback(() => {
    if (isExternal) {
      // After login, cookies are already set on the shared domain — navigate directly
      window.location.href = redirectUrl
    } else {
      navigate(redirectUrl)
    }
  }, [isExternal, redirectUrl, navigate])

  // Handle initial page load — stale state, auto-redirect, already authenticated
  useEffect(() => {
    // "checked" flag means auth_redirect already tried and failed to authenticate us.
    // Just show the login form — do NOT clear isAuthenticated in the store, because
    // that would broadcast to other tabs via localStorage and log them out too.
    if (alreadyChecked) {
      return
    }

    // External redirect: always go through the server endpoint so it can refresh
    // the access_token cookie before redirecting to the child app.
    if (shouldServerRedirect) {
      const apiBase = config.apiUrl || ''
      window.location.href = `${apiBase}/v1/auth/redirect?url=${encodeURIComponent(redirectUrl)}`
      return
    }

    // Already authenticated with an internal redirect — navigate directly
    if (isAuthenticated) {
      doRedirect()
      return
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // Cross-tab sync — when another tab logs in, redirect this tab too.
  // Skip when alreadyChecked: the server confirmed we can't authenticate via
  // cookies, so a stale isAuthenticated in localStorage must not re-trigger
  // the redirect and cause an infinite loop.
  const mountedRef = useRef(false)
  useEffect(() => {
    if (!mountedRef.current) {
      mountedRef.current = true
      return
    }
    if (isAuthenticated && !alreadyChecked) {
      doRedirect()
    }
  }, [isAuthenticated]) // eslint-disable-line react-hooks/exhaustive-deps

  // Show spinner while redirecting to API check
  if (shouldServerRedirect) {
    return (
      <div className="flex items-center justify-center min-h-[calc(100vh-8rem)]">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    )
  }

  const onSubmit = async (data: LoginFormData) => {
    setIsLoading(true)
    clearError()
    try {
      await login(data.email, data.password, data.remember)
      // Check if 2FA is required
      const { pendingChallenge } = useAuthStore.getState()
      if (pendingChallenge) {
        const params = redirectUrl !== '/dashboard' ? `?redirect=${encodeURIComponent(redirectUrl)}` : ''
        navigate(`/login/2fa${params}`)
      } else {
        doRedirect()
      }
    } catch {
      // Error is handled by the store
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">Welcome back</CardTitle>
          <CardDescription>
            Sign in to your account to continue
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="you@example.com"
                {...register('email')}
              />
              {errors.email && (
                <p className="text-sm text-destructive">{errors.email.message}</p>
              )}
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="password">Password</Label>
                <Link
                  to="/password-reset"
                  className="text-sm text-primary hover:underline"
                >
                  Forgot password?
                </Link>
              </div>
              <Input
                id="password"
                type="password"
                {...register('password')}
              />
              {errors.password && (
                <p className="text-sm text-destructive">{errors.password.message}</p>
              )}
            </div>

            <div className="flex items-center space-x-2">
              <input
                id="remember"
                type="checkbox"
                className="h-4 w-4 rounded border-border"
                {...register('remember')}
              />
              <Label htmlFor="remember" className="text-sm font-normal cursor-pointer">
                Remember me for 30 days
              </Label>
            </div>

            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Sign In
            </Button>
          </form>

          <div className="mt-6">
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <span className="w-full border-t" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-background px-2 text-muted-foreground">
                  Or continue with
                </span>
              </div>
            </div>

            <Link to="/magic-link" className="mt-4 block">
              <Button variant="outline" className="w-full">
                Sign in with Magic Link
              </Button>
            </Link>
          </div>

          <p className="mt-6 text-center text-sm text-muted-foreground">
            Don't have an account?{' '}
            <Link to="/register" className="text-primary hover:underline">
              Sign up
            </Link>
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
