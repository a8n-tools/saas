import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useAuthStore } from '@/stores/authStore'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { User, Lock, Shield, Check, Loader2, AlertCircle } from 'lucide-react'

const passwordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z
    .string()
    .min(12, 'Password must be at least 12 characters')
    .regex(/[a-z]/, 'Password must contain a lowercase letter')
    .regex(/[A-Z]/, 'Password must contain an uppercase letter')
    .regex(/[0-9]/, 'Password must contain a number'),
  confirmPassword: z.string(),
}).refine((data) => data.newPassword === data.confirmPassword, {
  message: "Passwords don't match",
  path: ['confirmPassword'],
})

type PasswordFormData = z.infer<typeof passwordSchema>

const passwordRequirements = [
  { label: 'At least 12 characters', test: (p: string) => p.length >= 12 },
  { label: 'One lowercase letter', test: (p: string) => /[a-z]/.test(p) },
  { label: 'One uppercase letter', test: (p: string) => /[A-Z]/.test(p) },
  { label: 'One number', test: (p: string) => /[0-9]/.test(p) },
]

export function SettingsPage() {
  const { user } = useAuthStore()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors },
  } = useForm<PasswordFormData>({
    resolver: zodResolver(passwordSchema),
  })

  const newPassword = watch('newPassword', '')

  const onSubmit = async (_data: PasswordFormData) => {
    setIsLoading(true)
    setError(null)
    setSuccess(false)
    try {
      // TODO: Implement password change API call
      await new Promise((resolve) => setTimeout(resolve, 1000))
      setSuccess(true)
      reset()
    } catch {
      setError('Failed to update password')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Settings</h1>
        <p className="mt-2 text-muted-foreground">
          Manage your account settings and preferences.
        </p>
      </div>

      {/* Account Info */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <User className="h-5 w-5 text-primary" />
            <CardTitle>Account Information</CardTitle>
          </div>
          <CardDescription>Your account details.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <Label className="text-muted-foreground">Email</Label>
              <p className="font-medium">{user?.email}</p>
            </div>
            <div>
              <Label className="text-muted-foreground">Account Type</Label>
              <p className="font-medium capitalize flex items-center gap-2">
                {user?.role}
                {user?.role === 'admin' && (
                  <Badge variant="default">Admin</Badge>
                )}
              </p>
            </div>
            <div>
              <Label className="text-muted-foreground">Email Verified</Label>
              <p className="font-medium flex items-center gap-2">
                {user?.email_verified ? (
                  <>
                    <Check className="h-4 w-4 text-green-600" />
                    Verified
                  </>
                ) : (
                  <>
                    <AlertCircle className="h-4 w-4 text-yellow-600" />
                    Not Verified
                  </>
                )}
              </p>
            </div>
            <div>
              <Label className="text-muted-foreground">Subscription Status</Label>
              <p className="font-medium capitalize">
                {user?.subscription_status || 'None'}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Change Password */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <Lock className="h-5 w-5 text-primary" />
            <CardTitle>Change Password</CardTitle>
          </div>
          <CardDescription>
            Update your password to keep your account secure.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 max-w-md">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            {success && (
              <Alert>
                <Check className="h-4 w-4" />
                <AlertDescription>Password updated successfully!</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="currentPassword">Current Password</Label>
              <Input
                id="currentPassword"
                type="password"
                {...register('currentPassword')}
              />
              {errors.currentPassword && (
                <p className="text-sm text-destructive">
                  {errors.currentPassword.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="newPassword">New Password</Label>
              <Input
                id="newPassword"
                type="password"
                {...register('newPassword')}
              />
              {errors.newPassword && (
                <p className="text-sm text-destructive">
                  {errors.newPassword.message}
                </p>
              )}
              <div className="mt-2 space-y-1">
                {passwordRequirements.map((req) => (
                  <div
                    key={req.label}
                    className={`flex items-center gap-2 text-xs ${
                      req.test(newPassword)
                        ? 'text-green-600'
                        : 'text-muted-foreground'
                    }`}
                  >
                    <Check
                      className={`h-3 w-3 ${
                        req.test(newPassword) ? 'opacity-100' : 'opacity-30'
                      }`}
                    />
                    {req.label}
                  </div>
                ))}
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="confirmPassword">Confirm New Password</Label>
              <Input
                id="confirmPassword"
                type="password"
                {...register('confirmPassword')}
              />
              {errors.confirmPassword && (
                <p className="text-sm text-destructive">
                  {errors.confirmPassword.message}
                </p>
              )}
            </div>

            <Button type="submit" disabled={isLoading}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Update Password
            </Button>
          </form>
        </CardContent>
      </Card>

      {/* Security */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <Shield className="h-5 w-5 text-primary" />
            <CardTitle>Security</CardTitle>
          </div>
          <CardDescription>
            Security settings and active sessions.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">
            Session management and two-factor authentication coming soon.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
