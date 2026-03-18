import { useState, useEffect } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { authApi } from '@/api'
import { useAuthStore } from '@/stores/authStore'
import type { AuthResponse } from '@/types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2, Shield } from 'lucide-react'

const passwordSchema = z.object({
  password: z
    .string()
    .min(12, 'Password must be at least 12 characters')
    .regex(/[a-z]/, 'Must contain a lowercase letter')
    .regex(/[A-Z]/, 'Must contain an uppercase letter')
    .regex(/[0-9]/, 'Must contain a digit')
    .regex(/[^a-zA-Z0-9]/, 'Must contain a special character'),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
})

type PasswordFormData = z.infer<typeof passwordSchema>

export function AcceptInvitePage() {
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [needsPassword, setNeedsPassword] = useState(false)
  const [email, setEmail] = useState<string | null>(null)

  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<PasswordFormData>({
    resolver: zodResolver(passwordSchema),
  })

  useEffect(() => {
    if (!token) {
      setError('No invite token provided')
      setIsLoading(false)
      return
    }

    const accept = async () => {
      try {
        const response = await authApi.acceptInvite({ token })
        if ('needs_password' in response && response.needs_password) {
          setNeedsPassword(true)
          setEmail(response.email)
        } else {
          const authResponse = response as AuthResponse
          useAuthStore.setState({
            user: authResponse.user,
            isAuthenticated: true,
            isLoading: false,
          })
          window.location.href = '/dashboard'
        }
      } catch (err) {
        const apiError = err as { error?: { message?: string } }
        setError(apiError.error?.message || 'Invalid or expired invite link')
      } finally {
        setIsLoading(false)
      }
    }
    accept()
  }, [token])

  const onSubmitPassword = async (data: PasswordFormData) => {
    if (!token) return
    setError(null)
    try {
      const response = await authApi.acceptInvite({ token, password: data.password })
      if ('needs_password' in response && response.needs_password) {
        setError('Password is required')
        return
      }
      const authResponse = response as AuthResponse
      useAuthStore.setState({
        user: authResponse.user,
        isAuthenticated: true,
        isLoading: false,
      })
      window.location.href = '/dashboard'
    } catch (err) {
      const apiError = err as { error?: { message?: string } }
      setError(apiError.error?.message || 'Failed to accept invite')
    }
  }

  if (isLoading) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardContent className="flex flex-col items-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
            <p className="mt-4 text-muted-foreground">Verifying invite...</p>
          </CardContent>
        </Card>
      </div>
    )
  }

  if (error && !needsPassword) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
              <AlertCircle className="h-6 w-6 text-destructive" />
            </div>
            <CardTitle className="text-2xl">Invitation Failed</CardTitle>
            <CardDescription>{error}</CardDescription>
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

  if (needsPassword) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
              <Shield className="h-6 w-6 text-primary" />
            </div>
            <CardTitle className="text-2xl">Create Your Account</CardTitle>
            <CardDescription>
              Set a password for <strong>{email}</strong> to complete your admin account setup.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit(onSubmitPassword)} className="space-y-4">
              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  placeholder="At least 12 characters"
                  {...register('password')}
                />
                {errors.password && (
                  <p className="text-sm text-destructive">{errors.password.message}</p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="confirmPassword">Confirm Password</Label>
                <Input
                  id="confirmPassword"
                  type="password"
                  placeholder="Re-enter your password"
                  {...register('confirmPassword')}
                />
                {errors.confirmPassword && (
                  <p className="text-sm text-destructive">{errors.confirmPassword.message}</p>
                )}
              </div>

              <ul className="text-xs text-muted-foreground space-y-1">
                <li>At least 12 characters</li>
                <li>Upper and lowercase letters</li>
                <li>At least one digit and one special character</li>
              </ul>

              <Button type="submit" className="w-full" disabled={isSubmitting}>
                {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create Account
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    )
  }

  return null
}
