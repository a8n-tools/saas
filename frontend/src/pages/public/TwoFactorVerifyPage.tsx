import { useState, useRef } from 'react'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2, Shield, ArrowLeft } from 'lucide-react'

function getRedirectUrl(params: URLSearchParams): string {
  const redirect = params.get('redirect')
  if (!redirect) return '/dashboard'
  if (redirect.startsWith('/') && !redirect.startsWith('//')) {
    return redirect
  }
  // External URLs — pass through (user just authenticated via login form,
  // so cookies are fresh and the redirect will work)
  return redirect
}

export function TwoFactorVerifyPage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const { pendingChallenge, verify2FA, error, clearError } = useAuthStore()
  const [code, setCode] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [useRecovery, setUseRecovery] = useState(false)
  const redirectUrl = getRedirectUrl(searchParams)
  const redirectingRef = useRef(false)

  const loginPath = redirectUrl !== '/dashboard'
    ? `/login?redirect=${encodeURIComponent(redirectUrl)}`
    : '/login'

  const doRedirect = () => {
    redirectingRef.current = true
    if (redirectUrl.startsWith('http')) {
      window.location.href = redirectUrl
    } else {
      navigate(redirectUrl)
    }
  }

  // Redirect if no pending challenge (but not if we're mid-redirect after successful 2FA)
  if (!pendingChallenge && !redirectingRef.current) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <CardTitle className="text-2xl">No pending verification</CardTitle>
            <CardDescription>Please log in first.</CardDescription>
          </CardHeader>
          <CardContent>
            <Link to={loginPath}>
              <Button className="w-full">Go to Login</Button>
            </Link>
          </CardContent>
        </Card>
      </div>
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!code.trim()) return
    setIsLoading(true)
    clearError()
    try {
      await verify2FA(code.trim())
      doRedirect()
    } catch {
      // Error is handled by the store
    } finally {
      setIsLoading(false)
    }
  }

  const handleCodeChange = (value: string) => {
    setCode(value)
    // Auto-submit on 6 digits for TOTP codes
    if (!useRecovery && value.replace(/\s/g, '').length === 6) {
      setIsLoading(true)
      clearError()
      verify2FA(value.trim())
        .then(() => doRedirect())
        .catch(() => {})
        .finally(() => setIsLoading(false))
    }
  }

  return (
    <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <Shield className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-2xl">Two-Factor Authentication</CardTitle>
          <CardDescription>
            {useRecovery
              ? 'Enter one of your recovery codes'
              : 'Enter the 6-digit code from your authenticator app'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="code">
                {useRecovery ? 'Recovery Code' : 'Authentication Code'}
              </Label>
              <Input
                id="code"
                type="text"
                inputMode={useRecovery ? 'text' : 'numeric'}
                placeholder={useRecovery ? 'XXXX-XXXX' : '000000'}
                value={code}
                onChange={(e) => handleCodeChange(e.target.value)}
                autoComplete="one-time-code"
                autoFocus
                className="text-center text-lg tracking-widest"
              />
            </div>

            <Button type="submit" className="w-full" disabled={isLoading || !code.trim()}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Verify
            </Button>
          </form>

          <div className="mt-4 text-center">
            <button
              type="button"
              onClick={() => {
                setUseRecovery(!useRecovery)
                setCode('')
                clearError()
              }}
              className="text-sm text-primary hover:underline"
            >
              {useRecovery
                ? 'Use authenticator app instead'
                : 'Use a recovery code instead'}
            </button>
          </div>

          <div className="mt-4 text-center">
            <Link
              to={loginPath}
              onClick={() => useAuthStore.getState().clearPendingChallenge()}
              className="inline-flex items-center text-sm text-muted-foreground hover:text-foreground"
            >
              <ArrowLeft className="mr-1 h-3 w-3" />
              Back to login
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
