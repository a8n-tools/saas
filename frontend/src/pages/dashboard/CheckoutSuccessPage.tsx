import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { authApi } from '@/api'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { CheckCircle, AppWindow, CreditCard, ArrowRight } from 'lucide-react'

export function CheckoutSuccessPage() {
  const navigate = useNavigate()
  const { user } = useAuthStore()
  const [countdown, setCountdown] = useState(10)
  const [ready, setReady] = useState(false)

  // Force a token refresh to get a new JWT with updated membership claims.
  // The old JWT still has membership_status: "none" even though the webhook
  // has already updated the DB. The refresh endpoint sets a new cookie but
  // doesn't return user data, so we fetch /users/me after.
  useEffect(() => {
    let cancelled = false
    const refresh = async () => {
      try {
        await authApi.refresh()
        // New JWT cookie is now set — fetch user with fresh claims
        const user = await authApi.me()
        if (!cancelled) {
          useAuthStore.getState().setUser(user)
          setReady(true)
        }
      } catch {
        if (!cancelled) setReady(true)
      }
    }
    refresh()
    return () => { cancelled = true }
  }, [])

  // Auto-redirect countdown (only starts after token is refreshed)
  useEffect(() => {
    if (!ready) return

    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(timer)
          navigate('/applications')
          return 0
        }
        return prev - 1
      })
    }, 1000)

    return () => clearInterval(timer)
  }, [ready, navigate])

  return (
    <div className="flex items-center justify-center min-h-[70vh]">
      <Card className="max-w-lg w-full border-border/50 overflow-hidden">
        <div className="h-1 bg-gradient-to-r from-teal-500 via-indigo-500 to-primary" />
        <CardContent className="pt-8 pb-8 text-center space-y-6">
          <div className="flex justify-center">
            <div className="rounded-full bg-gradient-to-br from-teal-500/20 to-teal-500/5 p-4">
              <CheckCircle className="h-12 w-12 text-teal-500" />
            </div>
          </div>

          <div className="space-y-2">
            <h1 className="text-3xl font-bold">
              Welcome aboard
              <span className="text-gradient bg-gradient-to-r from-primary to-indigo-500">!</span>
            </h1>
            <p className="text-muted-foreground text-lg">
              Your membership is now active. You have full access to all applications.
            </p>
          </div>

          {user && (
            <div className="bg-gradient-to-r from-indigo-500/5 via-primary/5 to-teal-500/5 rounded-lg p-4 space-y-2 text-sm border border-border/50">
              <div className="flex items-center justify-center gap-2">
                <CreditCard className="h-4 w-4 text-indigo-500" />
                <span className="capitalize font-medium">{user.membership_tier || 'Personal'} Plan</span>
              </div>
              <p className="text-muted-foreground">
                Your price is locked in for life — it will never increase.
              </p>
            </div>
          )}

          <div className="flex flex-col gap-3 pt-2">
            <Button size="lg" onClick={() => navigate('/applications')} className="gap-2 bg-gradient-to-r from-primary to-indigo-500 text-white border-0 shadow-lg shadow-primary/20">
              <AppWindow className="h-4 w-4" />
              Browse Applications
              <ArrowRight className="h-4 w-4" />
            </Button>
            <Button variant="outline" onClick={() => navigate('/membership')} className="border-indigo-300/30 text-indigo-600 hover:bg-indigo-500/10 dark:border-indigo-500/30 dark:text-indigo-400">
              View Membership Details
            </Button>
          </div>

          <p className="text-xs text-muted-foreground">
            Redirecting to applications in {countdown}s
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
