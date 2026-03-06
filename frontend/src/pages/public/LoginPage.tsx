import { useState, useCallback, useEffect } from 'react'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useAuthStore } from '@/stores/authStore'
import { authApi } from '@/api'
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
})

type LoginFormData = z.infer<typeof loginSchema>

function getBaseDomain(hostname: string): string {
  const parts = hostname.split('.')
  return parts.length >= 2 ? parts.slice(-2).join('.') : hostname
}

function isAllowedRedirect(redirectUrl: string): boolean {
  try {
    const url = new URL(redirectUrl)
    const domain = config.appDomain || getBaseDomain(window.location.hostname)
    return url.hostname === domain || url.hostname.endsWith(`.${domain}`)
  } catch {
    return false
  }
}

function getRedirectUrl(params: URLSearchParams): string {
  const redirect = params.get('redirect')
  if (!redirect) return '/dashboard'
  // Allow relative paths
  if (redirect.startsWith('/') && !redirect.startsWith('//')) {
    return redirect
  }
  // Allow full URLs on the same domain (e.g. https://go.example.com/...)
  if (isAllowedRedirect(redirect)) {
    return redirect
  }
  return '/dashboard'
}

export function LoginPage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const { login, isAuthenticated, error, clearError } = useAuthStore()
  const [isLoading, setIsLoading] = useState(false)
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  })

  const redirectUrl = getRedirectUrl(searchParams)
  const isExternal = redirectUrl.startsWith('http')
  const hasRedirect = searchParams.has('redirect')
  const [checkingSession, setCheckingSession] = useState(hasRedirect)

  const doRedirect = useCallback(() => {
    if (isExternal) {
      window.location.href = redirectUrl
    } else {
      navigate(redirectUrl)
    }
  }, [isExternal, redirectUrl, navigate])

  // When a redirect param is present, check if the user has a valid session
  // (they may have cookies but isAuthenticated=false in localStorage)
  useEffect(() => {
    if (!hasRedirect) return
    if (isAuthenticated) {
      doRedirect()
      return
    }
    // refreshUser() bails early if isAuthenticated is false, so call the API directly
    authApi.me()
      .then((user) => {
        useAuthStore.getState().setUser(user)
      })
      .catch(() => {
        // No valid session — show login form
      })
      .finally(() => setCheckingSession(false))
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // After session check or store hydration, redirect if authenticated
  useEffect(() => {
    if (isAuthenticated && hasRedirect) {
      doRedirect()
    }
  }, [isAuthenticated, hasRedirect, doRedirect])

  if (isAuthenticated || checkingSession) {
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
      await login(data.email, data.password)
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
