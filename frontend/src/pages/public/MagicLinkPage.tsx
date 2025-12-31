import { useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { authApi } from '@/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2, Mail, CheckCircle } from 'lucide-react'

const magicLinkSchema = z.object({
  email: z.string().email('Invalid email address'),
})

type MagicLinkFormData = z.infer<typeof magicLinkSchema>

export function MagicLinkPage() {
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<MagicLinkFormData>({
    resolver: zodResolver(magicLinkSchema),
  })

  const onSubmit = async (data: MagicLinkFormData) => {
    setIsLoading(true)
    setError(null)
    try {
      await authApi.requestMagicLink({ email: data.email })
      setSuccess(true)
    } catch (err) {
      const apiError = err as { error?: { message?: string } }
      setError(apiError.error?.message || 'Failed to send magic link')
    } finally {
      setIsLoading(false)
    }
  }

  // If there's a token in the URL, we're verifying
  if (token) {
    return <MagicLinkVerify token={token} />
  }

  // Show success message
  if (success) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-green-100">
              <CheckCircle className="h-6 w-6 text-green-600" />
            </div>
            <CardTitle className="text-2xl">Check your email</CardTitle>
            <CardDescription>
              We've sent a magic link to your email address. Click the link to
              sign in.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-center text-sm text-muted-foreground">
              The link will expire in 15 minutes.
            </p>
            <Link to="/login" className="mt-6 block">
              <Button variant="outline" className="w-full">
                Back to Login
              </Button>
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
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <Mail className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-2xl">Sign in with Magic Link</CardTitle>
          <CardDescription>
            We'll email you a link to sign in without a password.
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

            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Send Magic Link
            </Button>
          </form>

          <p className="mt-6 text-center text-sm text-muted-foreground">
            <Link to="/login" className="text-primary hover:underline">
              Back to regular login
            </Link>
          </p>
        </CardContent>
      </Card>
    </div>
  )
}

function MagicLinkVerify({ token }: { token: string }) {
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useState(() => {
    const verify = async () => {
      try {
        await authApi.verifyMagicLink({ token })
        window.location.href = '/dashboard'
      } catch (err) {
        const apiError = err as { error?: { message?: string } }
        setError(apiError.error?.message || 'Invalid or expired magic link')
      } finally {
        setIsLoading(false)
      }
    }
    verify()
  })

  if (isLoading) {
    return (
      <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
        <Card className="w-full max-w-md">
          <CardContent className="flex flex-col items-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
            <p className="mt-4 text-muted-foreground">Verifying magic link...</p>
          </CardContent>
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
            <CardTitle className="text-2xl">Verification Failed</CardTitle>
            <CardDescription>{error}</CardDescription>
          </CardHeader>
          <CardContent>
            <Link to="/magic-link" className="block">
              <Button className="w-full">Request New Magic Link</Button>
            </Link>
          </CardContent>
        </Card>
      </div>
    )
  }

  return null
}
