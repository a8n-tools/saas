import { Link } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { useApplications } from '@/hooks/useApplications'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { AppWindow, CreditCard, ExternalLink, Loader2 } from 'lucide-react'

export function DashboardPage() {
  const { user } = useAuthStore()
  const { applications, isLoading } = useApplications()

  const hasActiveSubscription =
    user?.subscription_status === 'active' ||
    user?.subscription_status === 'past_due'

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">Welcome back!</h1>
        <p className="mt-2 text-muted-foreground">
          Here's an overview of your account and applications.
        </p>
      </div>

      {/* Subscription Status */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <CreditCard className="h-5 w-5 text-primary" />
              <CardTitle className="text-lg">Subscription</CardTitle>
            </div>
            <SubscriptionBadge status={user?.subscription_status} />
          </div>
        </CardHeader>
        <CardContent>
          {hasActiveSubscription ? (
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">
                  You have access to all applications.
                  {user?.price_locked && (
                    <span className="ml-2 text-green-600 font-medium">
                      Price locked at $3/month
                    </span>
                  )}
                </p>
              </div>
              <Link to="/subscription">
                <Button variant="outline" size="sm">
                  Manage
                </Button>
              </Link>
            </div>
          ) : (
            <div className="flex items-center justify-between">
              <p className="text-sm text-muted-foreground">
                Subscribe to access all applications.
              </p>
              <Link to="/subscription">
                <Button size="sm">Subscribe Now</Button>
              </Link>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Applications */}
      <div>
        <div className="flex items-center gap-3 mb-4">
          <AppWindow className="h-5 w-5 text-primary" />
          <h2 className="text-xl font-semibold">Your Applications</h2>
        </div>

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
          </div>
        ) : (
          <div className="grid gap-4 md:grid-cols-2">
            {applications.map((app) => (
              <Card key={app.id}>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-lg">{app.name}</CardTitle>
                    {app.is_maintenance ? (
                      <Badge variant="warning">Maintenance</Badge>
                    ) : app.is_active ? (
                      <Badge variant="success">Active</Badge>
                    ) : (
                      <Badge variant="secondary">Inactive</Badge>
                    )}
                  </div>
                  <CardDescription>{app.description}</CardDescription>
                </CardHeader>
                <CardContent>
                  {hasActiveSubscription && app.is_active && !app.is_maintenance ? (
                    <a
                      href={`https://${app.subdomain}`}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <Button className="w-full">
                        Open {app.name}
                        <ExternalLink className="ml-2 h-4 w-4" />
                      </Button>
                    </a>
                  ) : (
                    <Button className="w-full" disabled>
                      {!hasActiveSubscription
                        ? 'Subscription Required'
                        : app.is_maintenance
                        ? 'Under Maintenance'
                        : 'Not Available'}
                    </Button>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>
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
      return <Badge variant="outline">No Subscription</Badge>
  }
}
