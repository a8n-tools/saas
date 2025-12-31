import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/authStore'
import { useSubscription } from '@/hooks/useSubscription'
import { subscriptionApi } from '@/api'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { formatDate, formatCurrency } from '@/lib/utils'
import {
  CreditCard,
  Loader2,
  AlertTriangle,
  CheckCircle,
  Receipt,
} from 'lucide-react'

export function SubscriptionPage() {
  const { user } = useAuthStore()
  const {
    subscription,
    isLoading,
    startCheckout,
    cancel,
    reactivate,
    willCancel,
  } = useSubscription()
  const [actionLoading, setActionLoading] = useState(false)

  const { data: payments, isLoading: paymentsLoading } = useQuery({
    queryKey: ['payments'],
    queryFn: () => subscriptionApi.getPaymentHistory(),
  })

  const handleCheckout = async () => {
    setActionLoading(true)
    try {
      await startCheckout()
    } catch {
      // Error handled by hook
    } finally {
      setActionLoading(false)
    }
  }

  const handleCancel = async () => {
    if (!confirm('Are you sure you want to cancel your subscription?')) return
    setActionLoading(true)
    try {
      await cancel()
    } catch {
      // Error handled by hook
    } finally {
      setActionLoading(false)
    }
  }

  const handleReactivate = async () => {
    setActionLoading(true)
    try {
      await reactivate()
    } catch {
      // Error handled by hook
    } finally {
      setActionLoading(false)
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    )
  }

  const hasSubscription = subscription && subscription.status !== 'canceled'
  const isPastDue = subscription?.status === 'past_due'

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Subscription</h1>
        <p className="mt-2 text-muted-foreground">
          Manage your subscription and billing.
        </p>
      </div>

      {isPastDue && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Your payment failed. Please update your payment method within 30
            days to avoid losing access.
          </AlertDescription>
        </Alert>
      )}

      {/* Current Plan */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <CreditCard className="h-5 w-5 text-primary" />
              <CardTitle>Current Plan</CardTitle>
            </div>
            <SubscriptionBadge status={subscription?.status} />
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {hasSubscription ? (
            <>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <p className="text-sm text-muted-foreground">Plan</p>
                  <p className="font-medium">All Access</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Price</p>
                  <p className="font-medium">
                    $3/month
                    {user?.price_locked && (
                      <Badge variant="outline" className="ml-2">
                        Locked
                      </Badge>
                    )}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Current Period</p>
                  <p className="font-medium">
                    {formatDate(subscription.current_period_start)} -{' '}
                    {formatDate(subscription.current_period_end)}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Next Billing</p>
                  <p className="font-medium">
                    {willCancel
                      ? 'Canceled - ends ' +
                        formatDate(subscription.current_period_end)
                      : formatDate(subscription.current_period_end)}
                  </p>
                </div>
              </div>

              <div className="flex gap-4 pt-4">
                {willCancel ? (
                  <Button onClick={handleReactivate} disabled={actionLoading}>
                    {actionLoading && (
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    )}
                    Reactivate Subscription
                  </Button>
                ) : (
                  <Button
                    variant="outline"
                    onClick={handleCancel}
                    disabled={actionLoading}
                  >
                    {actionLoading && (
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    )}
                    Cancel Subscription
                  </Button>
                )}
              </div>
            </>
          ) : (
            <div className="text-center py-8">
              <CreditCard className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Active Subscription</h3>
              <p className="text-muted-foreground mb-6">
                Subscribe to access all applications for $3/month.
              </p>
              <Button onClick={handleCheckout} disabled={actionLoading}>
                {actionLoading && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                Subscribe Now
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Payment History */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <Receipt className="h-5 w-5 text-primary" />
            <CardTitle>Payment History</CardTitle>
          </div>
          <CardDescription>Your recent payments and invoices.</CardDescription>
        </CardHeader>
        <CardContent>
          {paymentsLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-primary" />
            </div>
          ) : payments?.items.length === 0 ? (
            <p className="text-center text-muted-foreground py-8">
              No payment history yet.
            </p>
          ) : (
            <div className="space-y-4">
              {payments?.items.map((payment) => (
                <div
                  key={payment.id}
                  className="flex items-center justify-between py-3 border-b last:border-0"
                >
                  <div className="flex items-center gap-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-green-100">
                      <CheckCircle className="h-5 w-5 text-green-600" />
                    </div>
                    <div>
                      <p className="font-medium">
                        {formatCurrency(payment.amount_cents, payment.currency)}
                      </p>
                      <p className="text-sm text-muted-foreground">
                        {formatDate(payment.created_at)}
                      </p>
                    </div>
                  </div>
                  {payment.invoice_pdf_url && (
                    <a
                      href={payment.invoice_pdf_url}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <Button variant="ghost" size="sm">
                        Download
                      </Button>
                    </a>
                  )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function SubscriptionBadge({ status }: { status?: string }) {
  switch (status) {
    case 'active':
      return <Badge variant="success">Active</Badge>
    case 'past_due':
      return <Badge variant="warning">Past Due</Badge>
    case 'canceled':
      return <Badge variant="destructive">Canceled</Badge>
    case 'incomplete':
      return <Badge variant="secondary">Incomplete</Badge>
    default:
      return <Badge variant="outline">None</Badge>
  }
}
