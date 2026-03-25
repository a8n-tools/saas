import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { loadStripe } from '@stripe/stripe-js'
import {
  Elements,
  PaymentElement,
  useStripe,
  useElements,
} from '@stripe/react-stripe-js'
import { useAuthStore } from '@/stores/authStore'
import { apiClient } from '@/api/client'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Loader2, Check, CheckCircle2, CreditCard } from 'lucide-react'

const STRIPE_PUBLISHABLE_KEY = import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY ?? ''
const stripeEnabled = STRIPE_PUBLISHABLE_KEY.length > 0
const stripePromise = stripeEnabled ? loadStripe(STRIPE_PUBLISHABLE_KEY) : null

const registerSchema = z
  .object({
    email: z.string().email('Invalid email address'),
    password: z
      .string()
      .min(12, 'Password must be at least 12 characters')
      .regex(/[a-z]/, 'Password must contain a lowercase letter')
      .regex(/[A-Z]/, 'Password must contain an uppercase letter')
      .regex(/[0-9]/, 'Password must contain a number')
      .regex(/[^a-zA-Z0-9]/, 'Password must contain a special character'),
    confirmPassword: z.string(),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ['confirmPassword'],
  })

type RegisterFormData = z.infer<typeof registerSchema>

const passwordRequirements = [
  { label: 'At least 12 characters', test: (p: string) => p.length >= 12 },
  { label: 'One lowercase letter', test: (p: string) => /[a-z]/.test(p) },
  { label: 'One uppercase letter', test: (p: string) => /[A-Z]/.test(p) },
  { label: 'One number', test: (p: string) => /[0-9]/.test(p) },
  { label: 'One special character', test: (p: string) => /[^a-zA-Z0-9]/.test(p) },
]

interface StepTwoProps {
  email: string
  password: string
  stripeCustomerId: string
  onSuccess: () => void
  onBack: () => void
}

function CardAuthForm({ email, password, stripeCustomerId, onSuccess, onBack }: StepTwoProps) {
  const stripe = useStripe()
  const elements = useElements()
  const { register: registerUser, error: storeError } = useAuthStore()
  const [isLoading, setIsLoading] = useState(false)
  const [stripeError, setStripeError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!stripe || !elements) return

    setIsLoading(true)
    setStripeError(null)

    try {
      const { setupIntent, error } = await stripe.confirmSetup({
        elements,
        confirmParams: { return_url: window.location.href },
        redirect: 'if_required',
      })

      if (error) {
        setStripeError(error.message ?? 'Card authorization failed. Please try again.')
        return
      }

      const paymentMethodId =
        typeof setupIntent.payment_method === 'string'
          ? setupIntent.payment_method
          : setupIntent.payment_method?.id

      if (!paymentMethodId) {
        setStripeError('Failed to authorize card. Please try again.')
        return
      }

      await registerUser(email, password, {
        stripe_customer_id: stripeCustomerId,
        payment_method_id: paymentMethodId,
      })

      onSuccess()
    } catch {
      // Registration errors are set in the auth store and read via storeError below
    } finally {
      setIsLoading(false)
    }
  }

  const displayError = stripeError ?? storeError

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {displayError && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{displayError}</AlertDescription>
        </Alert>
      )}

      <div className="rounded-lg border p-4">
        <PaymentElement />
      </div>

      <p className="text-center text-xs text-muted-foreground">
        Your card will not be charged. We verify cards to prevent abuse.
      </p>

      <Button type="submit" className="w-full" disabled={isLoading || !stripe}>
        {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        Create Account
      </Button>

      <Button
        type="button"
        variant="ghost"
        className="w-full"
        onClick={onBack}
        disabled={isLoading}
      >
        Back
      </Button>
    </form>
  )
}

export function RegisterPage() {
  const navigate = useNavigate()
  const { register: registerUser, error, clearError } = useAuthStore()
  const [step, setStep] = useState<1 | 2>(1)
  const [clientSecret, setClientSecret] = useState<string | null>(null)
  const [stripeCustomerId, setStripeCustomerId] = useState<string | null>(null)
  const [formData, setFormData] = useState<{ email: string; password: string } | null>(null)
  const [isStepLoading, setIsStepLoading] = useState(false)
  const [stepError, setStepError] = useState<string | null>(null)
  const [isSuccess, setIsSuccess] = useState(false)

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<RegisterFormData>({
    resolver: zodResolver(registerSchema),
  })

  const password = watch('password', '')

  const onStep1Submit = async (data: RegisterFormData) => {
    setIsStepLoading(true)
    setStepError(null)
    clearError()
    try {
      if (!stripeEnabled) {
        await registerUser(data.email, data.password)
        handleRegistrationSuccess()
        return
      }
      const response = await apiClient.post<{ client_secret: string; customer_id: string }>(
        '/billing/setup-intent',
        { email: data.email }
      )
      setClientSecret(response.client_secret)
      setStripeCustomerId(response.customer_id)
      setFormData({ email: data.email, password: data.password })
      setStep(2)
    } catch (err) {
      const e = err as { error?: { message?: string } }
      setStepError(e.error?.message ?? 'Failed to initialize payment. Please try again.')
    } finally {
      setIsStepLoading(false)
    }
  }

  const handleRegistrationSuccess = () => {
    setIsSuccess(true)
    setTimeout(() => navigate('/dashboard'), 1500)
  }

  return (
    <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center py-12">
      <Card className="w-full max-w-md">
        {isSuccess ? (
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-green-500/20">
              <CheckCircle2 className="h-6 w-6 text-green-500" />
            </div>
            <CardTitle className="text-2xl">Account created!</CardTitle>
            <CardDescription>Redirecting to your dashboard...</CardDescription>
          </CardHeader>
        ) : step === 1 ? (
          <>
            <CardHeader className="text-center">
              <CardTitle className="text-2xl">Create an account</CardTitle>
              <CardDescription>Get access to all tools for $3/month</CardDescription>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSubmit(onStep1Submit)} className="space-y-4">
                {(stepError ?? error) && (
                  <Alert variant="destructive">
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>{stepError ?? error}</AlertDescription>
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
                  <Label htmlFor="password">Password</Label>
                  <Input id="password" type="password" {...register('password')} />
                  {errors.password && (
                    <p className="text-sm text-destructive">{errors.password.message}</p>
                  )}
                  <div className="mt-2 space-y-1">
                    {passwordRequirements.map((req) => (
                      <div
                        key={req.label}
                        className={`flex items-center gap-2 text-xs ${
                          req.test(password) ? 'text-green-600' : 'text-muted-foreground'
                        }`}
                      >
                        <Check
                          className={`h-3 w-3 ${req.test(password) ? 'opacity-100' : 'opacity-30'}`}
                        />
                        {req.label}
                      </div>
                    ))}
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="confirmPassword">Confirm Password</Label>
                  <Input id="confirmPassword" type="password" {...register('confirmPassword')} />
                  {errors.confirmPassword && (
                    <p className="text-sm text-destructive">{errors.confirmPassword.message}</p>
                  )}
                </div>

                <Button type="submit" className="w-full" disabled={isStepLoading}>
                  {isStepLoading ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : stripeEnabled ? (
                    <CreditCard className="mr-2 h-4 w-4" />
                  ) : null}
                  {stripeEnabled ? 'Continue to Card Verification' : 'Create Account'}
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
                    Sign up with Magic Link
                  </Button>
                </Link>
              </div>

              <p className="mt-6 text-center text-sm text-muted-foreground">
                Already have an account?{' '}
                <Link to="/login" className="text-primary hover:underline">
                  Sign in
                </Link>
              </p>

              <p className="mt-4 text-center text-xs text-muted-foreground">
                By creating an account, you agree to our{' '}
                <Link to="/terms" className="underline hover:text-foreground">
                  Terms of Service
                </Link>{' '}
                and{' '}
                <Link to="/privacy" className="underline hover:text-foreground">
                  Privacy Policy
                </Link>
                .
              </p>
            </CardContent>
          </>
        ) : (
          clientSecret &&
          formData &&
          stripeCustomerId && (
            <>
              <CardHeader className="text-center">
                <CardTitle className="text-2xl">Verify your card</CardTitle>
                <CardDescription>
                  A $0 authorization confirms your card is valid. You will not be charged now.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Elements stripe={stripePromise} options={{ clientSecret }}>
                  <CardAuthForm
                    email={formData.email}
                    password={formData.password}
                    stripeCustomerId={stripeCustomerId}
                    onSuccess={handleRegistrationSuccess}
                    onBack={() => setStep(1)}
                  />
                </Elements>
              </CardContent>
            </>
          )
        )}
      </Card>
    </div>
  )
}
