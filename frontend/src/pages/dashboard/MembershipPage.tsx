import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useMembership } from '@/hooks/useMembership'
import { membershipApi } from '@/api'
import { useAuthStore } from '@/stores/authStore'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { formatDate, formatCurrency } from '@/lib/utils'
import type { MembershipTier } from '@/types'
import {
  CreditCard,
  Loader2,
  AlertTriangle,
  CheckCircle,
  Receipt,
} from 'lucide-react'

// Helper to get tier display name
function getTierName(tier: MembershipTier | null | undefined): string {
  if (!tier) return 'Personal'
  return tier === 'business' ? 'Business' : 'Personal'
}

// Helper to get tier price
function getTierPrice(tier: MembershipTier | null | undefined, membership: { price_locked?: boolean, locked_price_amount?: number | null } | null): string {
  if (membership?.price_locked && membership?.locked_price_amount) {
    return `$${(membership.locked_price_amount / 100).toFixed(0)}/month`
  }
  if (tier === 'business') return '$15/month'
  return '$3/month'
}

export function MembershipPage() {
  const {
    membership,
    isLoading,
    startCheckout,
    cancel,
    reactivate,
    willCancel,
    tier,
  } = useMembership()
  const [actionLoading, setActionLoading] = useState(false)
  const [selectedTier, setSelectedTier] = useState<MembershipTier>('personal')

  const { data: payments, isLoading: paymentsLoading } = useQuery({
    queryKey: ['payments'],
    queryFn: () => membershipApi.getPaymentHistory(),
  })

  const handleSubscribe = async () => {
    setActionLoading(true)
    try {
      await startCheckout(selectedTier)
    } catch {
      // Error handled by hook
    } finally {
      setActionLoading(false)
    }
  }

  const handleCancel = async () => {
    if (!confirm('Are you sure you want to cancel your membership?')) return
    setActionLoading(true)
    try {
      await cancel()
      window.location.reload()
    } catch {
      // Error handled by hook
    } finally {
      setActionLoading(false)
    }
  }

  const handleCancelNow = async () => {
    if (!confirm('Cancel immediately? You will lose access right now.')) return
    setActionLoading(true)
    try {
      await membershipApi.cancelNow()
      await useAuthStore.getState().refreshUser()
      window.location.reload()
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

  const hasMembership = membership &&
    (membership.status === 'active' || membership.status === 'past_due')
  const isPastDue = membership?.status === 'past_due'

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Membership</h1>
        <p className="mt-2 text-muted-foreground">
          Manage your membership and billing.
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
            <MembershipBadge status={membership?.status} />
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {hasMembership ? (
            <>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <p className="text-sm text-muted-foreground">Plan</p>
                  <p className="font-medium">
                    {getTierName(tier)}
                    {tier === 'business' && (
                      <Badge variant="secondary" className="ml-2">Business</Badge>
                    )}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Price</p>
                  <p className="font-medium">
                    {getTierPrice(tier, membership)}
                    {membership?.price_locked && (
                      <Badge variant="outline" className="ml-2">
                        Locked
                      </Badge>
                    )}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Status</p>
                  <p className="font-medium capitalize">{membership.status}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Next Billing</p>
                  <p className="font-medium">
                    {willCancel
                      ? 'Canceled - ends ' +
                        (membership.current_period_end ? formatDate(membership.current_period_end) : 'N/A')
                      : membership.current_period_end ? formatDate(membership.current_period_end) : 'N/A'}
                  </p>
                </div>
              </div>

              <div className="flex gap-4 pt-4">
                {willCancel ? (
                  <Button onClick={handleReactivate} disabled={actionLoading}>
                    {actionLoading && (
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    )}
                    Reactivate Membership
                  </Button>
                ) : (
                  <>
                    <Button
                      variant="outline"
                      onClick={handleCancel}
                      disabled={actionLoading}
                    >
                      {actionLoading && (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      )}
                      Cancel Membership
                    </Button>
                    <Button
                      variant="destructive"
                      onClick={handleCancelNow}
                      disabled={actionLoading}
                    >
                      {actionLoading && (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      )}
                      Cancel Now
                    </Button>
                  </>
                )}
              </div>
            </>
          ) : (
            <div className="py-8">
              <div className="text-center mb-8">
                <CreditCard className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
                <h3 className="text-lg font-semibold mb-2">No Active Membership</h3>
                <p className="text-muted-foreground">
                  Subscribe to access all applications.
                </p>
              </div>

              {/* Tier Selection */}
              <div className="grid gap-4 md:grid-cols-2 mb-6">
                <button
                  type="button"
                  onClick={() => setSelectedTier('personal')}
                  className={`p-4 rounded-lg border-2 text-left transition-colors ${
                    selectedTier === 'personal'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-primary/50'
                  }`}
                >
                  <div className="font-semibold">Personal</div>
                  <div className="text-2xl font-bold mt-1">$3<span className="text-sm font-normal text-muted-foreground">/month</span></div>
                  <div className="text-sm text-muted-foreground mt-1">For individual developers</div>
                </button>
                <button
                  type="button"
                  onClick={() => setSelectedTier('business')}
                  className={`p-4 rounded-lg border-2 text-left transition-colors ${
                    selectedTier === 'business'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-primary/50'
                  }`}
                >
                  <div className="font-semibold">Business</div>
                  <div className="text-2xl font-bold mt-1">$15<span className="text-sm font-normal text-muted-foreground">/month</span></div>
                  <div className="text-sm text-muted-foreground mt-1">For teams and organizations</div>
                </button>
              </div>

              <div className="text-center">
                <Button onClick={handleSubscribe} disabled={actionLoading} size="lg">
                  {actionLoading && (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  Subscribe to {selectedTier === 'business' ? 'Business' : 'Personal'}
                </Button>
              </div>
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

function MembershipBadge({ status }: { status?: string }) {
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
