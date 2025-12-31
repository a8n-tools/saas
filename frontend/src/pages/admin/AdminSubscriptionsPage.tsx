import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { formatDate, formatCurrency } from '@/lib/utils'
import { CreditCard, MoreVertical } from 'lucide-react'

export function AdminSubscriptionsPage() {
  // TODO: Fetch actual subscriptions from API
  const subscriptions = [
    {
      id: '1',
      user_email: 'user1@example.com',
      status: 'active',
      current_period_end: '2025-01-15T00:00:00Z',
      amount_cents: 300,
    },
    {
      id: '2',
      user_email: 'user2@example.com',
      status: 'past_due',
      current_period_end: '2025-01-10T00:00:00Z',
      amount_cents: 300,
    },
    {
      id: '3',
      user_email: 'user3@example.com',
      status: 'canceled',
      current_period_end: '2024-12-20T00:00:00Z',
      amount_cents: 300,
    },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Subscriptions</h1>
        <p className="mt-2 text-muted-foreground">
          View and manage all subscriptions.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Active</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {subscriptions.filter((s) => s.status === 'active').length}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Past Due</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-yellow-600">
              {subscriptions.filter((s) => s.status === 'past_due').length}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Canceled</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {subscriptions.filter((s) => s.status === 'canceled').length}
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>All Subscriptions</CardTitle>
          <CardDescription>Manage customer subscriptions.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {subscriptions.map((sub) => (
              <div
                key={sub.id}
                className="flex items-center justify-between py-4 border-b last:border-0"
              >
                <div className="flex items-center gap-4">
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-muted">
                    <CreditCard className="h-5 w-5 text-muted-foreground" />
                  </div>
                  <div>
                    <p className="font-medium">{sub.user_email}</p>
                    <p className="text-sm text-muted-foreground">
                      {formatCurrency(sub.amount_cents)} • Ends{' '}
                      {formatDate(sub.current_period_end)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  <SubscriptionBadge status={sub.status} />
                  <Button variant="ghost" size="icon">
                    <MoreVertical className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function SubscriptionBadge({ status }: { status: string }) {
  switch (status) {
    case 'active':
      return <Badge variant="success">Active</Badge>
    case 'past_due':
      return <Badge variant="warning">Past Due</Badge>
    case 'canceled':
      return <Badge variant="destructive">Canceled</Badge>
    default:
      return <Badge variant="outline">Unknown</Badge>
  }
}
