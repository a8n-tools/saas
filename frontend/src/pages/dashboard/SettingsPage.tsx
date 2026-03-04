import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { authApi } from '@/api'
import { useAuthStore } from '@/stores/authStore'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { User, Lock, Shield, Check, Loader2, AlertCircle, Mail, ShieldCheck, ShieldOff, KeyRound } from 'lucide-react'

const passwordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z
    .string()
    .min(12, 'Password must be at least 12 characters')
    .regex(/[a-z]/, 'Password must contain a lowercase letter')
    .regex(/[A-Z]/, 'Password must contain an uppercase letter')
    .regex(/[0-9]/, 'Password must contain a number')
    .regex(/[^a-zA-Z0-9]/, 'Password must contain a special character'),
  confirmPassword: z.string(),
}).refine((data) => data.newPassword === data.confirmPassword, {
  message: "Passwords don't match",
  path: ['confirmPassword'],
})

type PasswordFormData = z.infer<typeof passwordSchema>

const emailSchema = z.object({
  newEmail: z.string().email('Invalid email address'),
  currentPassword: z.string().optional(),
})

type EmailFormData = z.infer<typeof emailSchema>

const passwordRequirements = [
  { label: 'At least 12 characters', test: (p: string) => p.length >= 12 },
  { label: 'One lowercase letter', test: (p: string) => /[a-z]/.test(p) },
  { label: 'One uppercase letter', test: (p: string) => /[A-Z]/.test(p) },
  { label: 'One number', test: (p: string) => /[0-9]/.test(p) },
  { label: 'One special character', test: (p: string) => /[^a-zA-Z0-9]/.test(p) },
]

export function SettingsPage() {
  const navigate = useNavigate()
  const { user, logout } = useAuthStore()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  // Email change state
  const [emailLoading, setEmailLoading] = useState(false)
  const [emailError, setEmailError] = useState<string | null>(null)
  const [emailSuccess, setEmailSuccess] = useState<string | null>(null)

  // Email verification state
  const [verifyLoading, setVerifyLoading] = useState(false)
  const [verifyError, setVerifyError] = useState<string | null>(null)
  const [verifySuccess, setVerifySuccess] = useState<string | null>(null)

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

  const {
    register: registerEmail,
    handleSubmit: handleEmailSubmit,
    reset: resetEmail,
    formState: { errors: emailErrors },
  } = useForm<EmailFormData>({
    resolver: zodResolver(emailSchema),
  })

  const onEmailSubmit = async (data: EmailFormData) => {
    setEmailLoading(true)
    setEmailError(null)
    setEmailSuccess(null)
    try {
      const result = await authApi.requestEmailChange({
        new_email: data.newEmail,
        current_password: data.currentPassword || undefined,
      })
      if (result.requires_relogin) {
        logout()
        navigate('/login')
        return
      }
      setEmailSuccess(result.message)
      resetEmail()
    } catch (err) {
      const apiError = err as { error?: { message?: string } }
      setEmailError(apiError.error?.message || 'Failed to update email')
    } finally {
      setEmailLoading(false)
    }
  }

  const onSubmit = async (data: PasswordFormData) => {
    setIsLoading(true)
    setError(null)
    setSuccess(false)
    try {
      await authApi.changePassword({
        current_password: data.currentPassword,
        new_password: data.newPassword,
      })
      setSuccess(true)
      reset()
    } catch (err) {
      const apiError = err as { error?: { message?: string } }
      setError(apiError.error?.message || 'Failed to update password')
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
      <Card className="border-border/50 overflow-hidden">
        <div className="h-1 bg-gradient-to-r from-primary via-indigo-500 to-teal-500" />
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-indigo-500">
              <User className="h-4 w-4 text-white" />
            </div>
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
              <div className="space-y-2">
                {user?.email_verified ? (
                  <p className="font-medium flex items-center gap-2">
                    <Check className="h-4 w-4 text-teal-600 dark:text-teal-400" />
                    Verified
                  </p>
                ) : (
                  <>
                    <p className="font-medium flex items-center gap-2">
                      <AlertCircle className="h-4 w-4 text-yellow-600" />
                      Not Verified
                    </p>
                    {user?.two_factor_enabled ? (
                      <>
                        {verifyError && (
                          <p className="text-sm text-destructive">{verifyError}</p>
                        )}
                        {verifySuccess && (
                          <p className="text-sm text-teal-600 dark:text-teal-400">{verifySuccess}</p>
                        )}
                        <Button
                          size="sm"
                          variant="outline"
                          disabled={verifyLoading}
                          onClick={async () => {
                            setVerifyLoading(true)
                            setVerifyError(null)
                            setVerifySuccess(null)
                            try {
                              const result = await authApi.requestEmailVerification()
                              setVerifySuccess(result.message)
                            } catch (err) {
                              const apiError = err as { error?: { message?: string } }
                              setVerifyError(apiError.error?.message || 'Failed to send verification email')
                            } finally {
                              setVerifyLoading(false)
                            }
                          }}
                        >
                          {verifyLoading && <Loader2 className="mr-2 h-3 w-3 animate-spin" />}
                          <Mail className="mr-2 h-3 w-3" />
                          Verify Email
                        </Button>
                      </>
                    ) : (
                      <p className="text-xs text-muted-foreground">
                        Enable 2FA to verify your email
                      </p>
                    )}
                  </>
                )}
              </div>
            </div>
            <div>
              <Label className="text-muted-foreground">Membership Status</Label>
              <p className="font-medium capitalize">
                {user?.membership_status || 'None'}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Change Email */}
      <Card className="border-border/50">
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-teal-500">
              <Mail className="h-4 w-4 text-white" />
            </div>
            <CardTitle>Change Email</CardTitle>
          </div>
          <CardDescription>
            Update your email address.{' '}
            {user?.email_verified
              ? 'A verification email will be sent to your new address.'
              : 'Your email will be updated immediately.'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleEmailSubmit(onEmailSubmit)} className="space-y-4 max-w-md">
            {emailError && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{emailError}</AlertDescription>
              </Alert>
            )}
            {emailSuccess && (
              <Alert>
                <Check className="h-4 w-4" />
                <AlertDescription>{emailSuccess}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="newEmail">New Email Address</Label>
              <Input
                id="newEmail"
                type="email"
                placeholder="Enter your new email"
                {...registerEmail('newEmail')}
              />
              {emailErrors.newEmail && (
                <p className="text-sm text-destructive">
                  {emailErrors.newEmail.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="emailCurrentPassword">Current Password</Label>
              <Input
                id="emailCurrentPassword"
                type="password"
                placeholder="Enter your current password"
                {...registerEmail('currentPassword')}
              />
              {emailErrors.currentPassword && (
                <p className="text-sm text-destructive">
                  {emailErrors.currentPassword.message}
                </p>
              )}
            </div>

            <Button type="submit" disabled={emailLoading} className="bg-gradient-to-r from-primary to-teal-500 text-white border-0">
              {emailLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Change Email
            </Button>
          </form>
        </CardContent>
      </Card>

      {/* Change Password */}
      <Card className="border-border/50">
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-indigo-500 to-teal-500">
              <Lock className="h-4 w-4 text-white" />
            </div>
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
                        ? 'text-teal-600 dark:text-teal-400'
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

            <Button type="submit" disabled={isLoading} className="bg-gradient-to-r from-primary to-indigo-500 text-white border-0">
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Update Password
            </Button>
          </form>
        </CardContent>
      </Card>

      {/* Security / Two-Factor Authentication */}
      <TwoFactorCard />
    </div>
  )
}

function TwoFactorCard() {
  const navigate = useNavigate()
  const { user } = useAuthStore()
  const [status, setStatus] = useState<{ enabled: boolean; recovery_codes_remaining: number } | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Disable dialog
  const [showDisable, setShowDisable] = useState(false)
  const [disablePassword, setDisablePassword] = useState('')
  const [disableLoading, setDisableLoading] = useState(false)

  // Regenerate dialog
  const [showRegenerate, setShowRegenerate] = useState(false)
  const [regenPassword, setRegenPassword] = useState('')
  const [regenLoading, setRegenLoading] = useState(false)
  const [newCodes, setNewCodes] = useState<string[] | null>(null)

  useEffect(() => {
    authApi.get2FAStatus()
      .then(setStatus)
      .catch(() => setStatus(null))
      .finally(() => setIsLoading(false))
  }, [])

  const handleDisable = async () => {
    setDisableLoading(true)
    setError(null)
    try {
      await authApi.disable2FA({ password: disablePassword })
      setStatus({ enabled: false, recovery_codes_remaining: 0 })
      setShowDisable(false)
      setDisablePassword('')
      setSuccess('Two-factor authentication has been disabled.')
      useAuthStore.getState().refreshUser()
    } catch (err) {
      const apiError = err as { error?: { message?: string } }
      setError(apiError.error?.message || 'Failed to disable 2FA')
    } finally {
      setDisableLoading(false)
    }
  }

  const handleRegenerate = async () => {
    setRegenLoading(true)
    setError(null)
    try {
      const response = await authApi.regenerateRecoveryCodes({ password: regenPassword })
      setNewCodes(response.codes)
      setRegenPassword('')
      // Update status count
      setStatus((prev) => prev ? { ...prev, recovery_codes_remaining: response.codes.length } : prev)
    } catch (err) {
      const apiError = err as { error?: { message?: string } }
      setError(apiError.error?.message || 'Failed to regenerate codes')
    } finally {
      setRegenLoading(false)
    }
  }

  const isAdmin = user?.role === 'admin'

  return (
    <>
      <Card className="border-border/50">
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-teal-500 to-indigo-500">
              <Shield className="h-4 w-4 text-white" />
            </div>
            <CardTitle>Two-Factor Authentication</CardTitle>
          </div>
          <CardDescription>
            Add an extra layer of security to your account with TOTP-based two-factor authentication.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          {success && (
            <Alert>
              <Check className="h-4 w-4" />
              <AlertDescription>{success}</AlertDescription>
            </Alert>
          )}

          {isLoading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Loading...
            </div>
          ) : status?.enabled ? (
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <ShieldCheck className="h-5 w-5 text-teal-600 dark:text-teal-400" />
                <span className="font-medium">Enabled</span>
                <Badge variant="default" className="bg-teal-600">Active</Badge>
              </div>

              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <KeyRound className="h-4 w-4" />
                <span>{status.recovery_codes_remaining} recovery codes remaining</span>
              </div>

              <div className="flex gap-3">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    setShowRegenerate(true)
                    setNewCodes(null)
                    setError(null)
                  }}
                >
                  <KeyRound className="mr-2 h-4 w-4" />
                  Regenerate Recovery Codes
                </Button>
                {!isAdmin && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="text-destructive hover:text-destructive"
                    onClick={() => {
                      setShowDisable(true)
                      setError(null)
                    }}
                  >
                    <ShieldOff className="mr-2 h-4 w-4" />
                    Disable 2FA
                  </Button>
                )}
              </div>
              {isAdmin && (
                <p className="text-xs text-muted-foreground">
                  Admin accounts cannot disable two-factor authentication.
                </p>
              )}
            </div>
          ) : (
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <ShieldOff className="h-5 w-5 text-muted-foreground" />
                <span className="font-medium text-muted-foreground">Not enabled</span>
              </div>
              <Button
                onClick={() => navigate('/settings/2fa/setup')}
                className="bg-gradient-to-r from-teal-500 to-indigo-500 text-white border-0"
              >
                <Shield className="mr-2 h-4 w-4" />
                Enable Two-Factor Authentication
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Disable 2FA Dialog */}
      <Dialog open={showDisable} onOpenChange={setShowDisable}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Disable Two-Factor Authentication</DialogTitle>
            <DialogDescription>
              Enter your password to confirm. This will remove the extra security from your account.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <div className="space-y-2">
              <Label htmlFor="disable-password">Password</Label>
              <Input
                id="disable-password"
                type="password"
                value={disablePassword}
                onChange={(e) => setDisablePassword(e.target.value)}
                autoFocus
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowDisable(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDisable}
              disabled={disableLoading || !disablePassword}
            >
              {disableLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Disable 2FA
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Regenerate Recovery Codes Dialog */}
      <Dialog open={showRegenerate} onOpenChange={(open) => {
        setShowRegenerate(open)
        if (!open) setNewCodes(null)
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {newCodes ? 'New Recovery Codes' : 'Regenerate Recovery Codes'}
            </DialogTitle>
            <DialogDescription>
              {newCodes
                ? 'Save these codes in a safe place. Your old codes are no longer valid.'
                : 'Enter your password to generate new recovery codes. This will invalidate all existing codes.'}
            </DialogDescription>
          </DialogHeader>
          {newCodes ? (
            <div className="space-y-4 py-4">
              <Alert>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  Save these codes now. You won't be able to see them again.
                </AlertDescription>
              </Alert>
              <div className="grid grid-cols-2 gap-2 rounded-lg bg-muted p-4">
                {newCodes.map((code, i) => (
                  <code key={i} className="text-center font-mono text-sm py-1">
                    {code}
                  </code>
                ))}
              </div>
              <Button
                variant="outline"
                className="w-full"
                onClick={() => navigator.clipboard.writeText(newCodes.join('\n'))}
              >
                Copy Codes
              </Button>
            </div>
          ) : (
            <div className="space-y-4 py-4">
              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}
              <div className="space-y-2">
                <Label htmlFor="regen-password">Password</Label>
                <Input
                  id="regen-password"
                  type="password"
                  value={regenPassword}
                  onChange={(e) => setRegenPassword(e.target.value)}
                  autoFocus
                />
              </div>
            </div>
          )}
          <DialogFooter>
            {newCodes ? (
              <Button onClick={() => setShowRegenerate(false)}>Done</Button>
            ) : (
              <>
                <Button variant="outline" onClick={() => setShowRegenerate(false)}>
                  Cancel
                </Button>
                <Button
                  onClick={handleRegenerate}
                  disabled={regenLoading || !regenPassword}
                >
                  {regenLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Regenerate
                </Button>
              </>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
