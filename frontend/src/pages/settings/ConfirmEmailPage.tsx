import { useEffect, useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { authApi } from '@/api'
import { useAuthStore } from '@/stores/authStore'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Check, Loader2, Mail } from 'lucide-react'

export function ConfirmEmailPage() {
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)
  const { logout } = useAuthStore()

  useEffect(() => {
    if (!token) return

    const confirmEmail = async () => {
      setIsLoading(true)
      setError(null)
      try {
        await authApi.confirmEmailChange({ token })
        setSuccess(true)
        // Clear auth state since sessions were revoked
        logout()
      } catch (err) {
        const apiError = err as { error?: { message?: string } }
        setError(apiError.error?.message || 'Failed to confirm email change')
      } finally {
        setIsLoading(false)
      }
    }

    confirmEmail()
  }, [token, logout])

  if (!token) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
              <AlertCircle className="h-6 w-6 text-destructive" />
            </div>
            <CardTitle className="text-2xl">Invalid Link</CardTitle>
            <CardDescription>
              This email confirmation link is invalid or has expired.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link to="/settings" className="block">
              <Button className="w-full">Go to Settings</Button>
            </Link>
          </CardContent>
        </Card>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
              <Loader2 className="h-6 w-6 text-primary animate-spin" />
            </div>
            <CardTitle className="text-2xl">Confirming email change...</CardTitle>
            <CardDescription>Please wait while we verify your new email address.</CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
              <AlertCircle className="h-6 w-6 text-destructive" />
            </div>
            <CardTitle className="text-2xl">Confirmation Failed</CardTitle>
            <CardDescription>We couldn't confirm your email change.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
            <Link to="/settings" className="block">
              <Button className="w-full">Go to Settings</Button>
            </Link>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-teal-500/10">
            <Check className="h-6 w-6 text-teal-600 dark:text-teal-400" />
          </div>
          <CardTitle className="text-2xl">Email Updated</CardTitle>
          <CardDescription>
            Your email address has been changed successfully. Please log in with your new email.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link to="/login" className="block">
            <Button className="w-full">Go to Login</Button>
          </Link>
        </CardContent>
      </Card>
    </div>
  )
}
