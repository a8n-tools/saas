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

  const hasActiveMembership =
    user?.membership_status === 'active' ||
    user?.membership_status === 'past_due'

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">Welcome back!</h1>
        <p className="mt-2 text-muted-foreground">
          Here's an overview of your account and applications.
        </p>
      </div>

      {/* Membership Status */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <CreditCard className="h-5 w-5 text-primary" />
              <CardTitle className="text-lg">Membership</CardTitle>
            </div>
            <MembershipBadge status={user?.membership_status} />
          </div>
        </CardHeader>
        <CardContent>
          {hasActiveMembership ? (
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
              <Link to="/membership">
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
              <Link to="/membership">
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
            {applications.map((app) => {
              const baseDomain = import.meta.env.VITE_APP_DOMAIN || 'a8n.tools'
              const appUrl = `${app.slug}.${baseDomain}`
              return (
                <Card key={app.id}>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-lg">{app.display_name}</CardTitle>
                      {app.maintenance_mode ? (
                        <Badge variant="warning">Maintenance</Badge>
                      ) : app.is_accessible ? (
                        <Badge variant="success">Active</Badge>
                      ) : (
                        <Badge variant="secondary">Locked</Badge>
                      )}
                    </div>
                    <CardDescription>{app.description}</CardDescription>
                  </CardHeader>
                  <CardContent>
                    {app.is_accessible ? (
                      <a
                        href={`https://${appUrl}`}
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        <Button className="w-full">
                          Open {app.display_name}
                          <ExternalLink className="ml-2 h-4 w-4" />
                        </Button>
                      </a>
                    ) : (
                      <Button className="w-full" disabled>
                        {!hasActiveMembership
                          ? 'Membership Required'
                          : app.maintenance_mode
                          ? 'Under Maintenance'
                          : 'Not Available'}
                      </Button>
                    )}
                  </CardContent>
                </Card>
              )
            })}
          </div>
        )}
      </div>
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
      return <Badge variant="outline">No Membership</Badge>
  }
}
