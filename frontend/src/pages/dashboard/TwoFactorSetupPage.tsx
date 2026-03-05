import { useState, useEffect, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { QRCodeSVG } from 'qrcode.react'
import { authApi } from '@/api'
import { useAuthStore } from '@/stores/authStore'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2, Shield, Copy, Check, Download } from 'lucide-react'

type SetupStep = 'scan' | 'verify' | 'recovery'

export function TwoFactorSetupPage() {
  const navigate = useNavigate()
  const [step, setStep] = useState<SetupStep>('scan')
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Setup data
  const [otpauthUri, setOtpauthUri] = useState('')
  const [secret, setSecret] = useState('')
  const [verificationCode, setVerificationCode] = useState('')
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([])
  const [copied, setCopied] = useState(false)
  const [acknowledged, setAcknowledged] = useState(false)

  const startSetup = () => {
    setIsLoading(true)
    setError(null)
    authApi.setup2FA()
      .then((response) => {
        setOtpauthUri(response.otpauth_uri)
        setSecret(response.secret)
        setStep('scan')
      })
      .catch((err) => {
        const apiError = err as { error?: { message?: string } }
        setError(apiError.error?.message || 'Failed to start 2FA setup')
      })
      .finally(() => {
        setIsLoading(false)
      })
  }

  const setupStarted = useRef(false)

  useEffect(() => {
    if (setupStarted.current) return
    setupStarted.current = true
    startSetup()
  }, [])

  const confirmSetup = async () => {
    if (!verificationCode.trim()) return
    setIsLoading(true)
    setError(null)
    try {
      const response = await authApi.confirm2FA({ code: verificationCode.trim() })
      setRecoveryCodes(response.codes)
      setStep('recovery')
      // Refresh user data to get updated two_factor_enabled
      useAuthStore.getState().refreshUser()
    } catch (err) {
      const apiError = err as { error?: { message?: string } }
      setError(apiError.error?.message || 'Invalid verification code')
    } finally {
      setIsLoading(false)
    }
  }

  const copySecret = async () => {
    await navigator.clipboard.writeText(secret)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const copyCodes = async () => {
    await navigator.clipboard.writeText(recoveryCodes.join('\n'))
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const downloadCodes = () => {
    const text = `Recovery Codes\n${'='.repeat(30)}\n\nStore these codes in a safe place. Each code can only be used once.\n\n${recoveryCodes.join('\n')}\n`
    const blob = new Blob([text], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'recovery-codes.txt'
    a.click()
    URL.revokeObjectURL(url)
  }

  if (step === 'scan') {
    return (
      <div className="mx-auto max-w-lg space-y-6">
        <div>
          <h1 className="text-3xl font-bold">Set Up Two-Factor Authentication</h1>
          <p className="mt-2 text-muted-foreground">
            Add an extra layer of security to your account.
          </p>
        </div>

        <Card className="border-border/50">
          <CardHeader>
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-indigo-500">
                <Shield className="h-4 w-4 text-white" />
              </div>
              <div>
                <CardTitle>Step 1: Scan QR Code</CardTitle>
                <CardDescription>
                  Scan this QR code with your authenticator app (Google Authenticator, Authy, etc.)
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-6">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            {isLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
              </div>
            ) : otpauthUri ? (
              <>
                <div className="flex justify-center rounded-lg bg-white p-4">
                  <QRCodeSVG value={otpauthUri} size={200} />
                </div>

                <div className="space-y-2">
                  <Label className="text-muted-foreground">
                    Or enter this key manually:
                  </Label>
                  <div className="flex items-center gap-2">
                    <code className="flex-1 rounded bg-muted px-3 py-2 text-sm font-mono break-all">
                      {secret}
                    </code>
                    <Button variant="outline" size="icon" onClick={copySecret}>
                      {copied ? (
                        <Check className="h-4 w-4 text-teal-600" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                </div>

                <Button
                  className="w-full"
                  onClick={() => {
                    setStep('verify')
                    setError(null)
                  }}
                >
                  Continue
                </Button>
              </>
            ) : (
              <Button variant="outline" onClick={startSetup}>
                Try Again
              </Button>
            )}
          </CardContent>
        </Card>
      </div>
    )
  }

  if (step === 'verify') {
    return (
      <div className="mx-auto max-w-lg space-y-6">
        <div>
          <h1 className="text-3xl font-bold">Set Up Two-Factor Authentication</h1>
          <p className="mt-2 text-muted-foreground">
            Verify your authenticator app is working correctly.
          </p>
        </div>

        <Card className="border-border/50">
          <CardHeader>
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-indigo-500">
                <Shield className="h-4 w-4 text-white" />
              </div>
              <div>
                <CardTitle>Step 2: Verify Code</CardTitle>
                <CardDescription>
                  Enter the 6-digit code from your authenticator app to confirm setup.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="verification-code">Verification Code</Label>
              <Input
                id="verification-code"
                type="text"
                inputMode="numeric"
                placeholder="000000"
                value={verificationCode}
                onChange={(e) => setVerificationCode(e.target.value)}
                autoComplete="one-time-code"
                autoFocus
                className="text-center text-lg tracking-widest"
              />
            </div>

            <div className="flex gap-3">
              <Button
                variant="outline"
                onClick={() => {
                  setStep('scan')
                  setError(null)
                }}
              >
                Back
              </Button>
              <Button
                className="flex-1"
                onClick={confirmSetup}
                disabled={isLoading || !verificationCode.trim()}
              >
                {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Verify & Enable
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  // Recovery codes step
  return (
    <div className="mx-auto max-w-lg space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Two-Factor Authentication Enabled</h1>
        <p className="mt-2 text-muted-foreground">
          Save your recovery codes in a safe place.
        </p>
      </div>

      <Card className="border-border/50">
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-teal-500 to-indigo-500">
              <Shield className="h-4 w-4 text-white" />
            </div>
            <div>
              <CardTitle>Step 3: Save Recovery Codes</CardTitle>
              <CardDescription>
                If you lose access to your authenticator app, you can use these codes to sign in.
                Each code can only be used once.
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Save these codes now. You won't be able to see them again.
            </AlertDescription>
          </Alert>

          <div className="grid grid-cols-2 gap-2 rounded-lg bg-muted p-4">
            {recoveryCodes.map((code, i) => (
              <code key={i} className="text-center font-mono text-sm py-1">
                {code}
              </code>
            ))}
          </div>

          <div className="flex gap-2">
            <Button variant="outline" className="flex-1" onClick={copyCodes}>
              {copied ? (
                <Check className="mr-2 h-4 w-4 text-teal-600" />
              ) : (
                <Copy className="mr-2 h-4 w-4" />
              )}
              Copy
            </Button>
            <Button variant="outline" className="flex-1" onClick={downloadCodes}>
              <Download className="mr-2 h-4 w-4" />
              Download
            </Button>
          </div>

          <div className="flex items-center gap-2 pt-2">
            <input
              type="checkbox"
              id="acknowledge"
              checked={acknowledged}
              onChange={(e) => setAcknowledged(e.target.checked)}
              className="rounded"
            />
            <Label htmlFor="acknowledge" className="text-sm cursor-pointer">
              I have saved my recovery codes
            </Label>
          </div>

          <Button
            className="w-full"
            disabled={!acknowledged}
            onClick={() => navigate('/settings')}
          >
            Done
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}
