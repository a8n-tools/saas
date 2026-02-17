import { Link } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { useApplications } from '@/hooks/useApplications'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { AppWindow, CreditCard, ExternalLink, Loader2, ArrowRight } from 'lucide-react'

export function DashboardPage() {
  const { user } = useAuthStore()
  const { applications, isLoading } = useApplications()

  const hasActiveMembership =
    user?.membership_status === 'active' ||
    user?.membership_status === 'past_due'

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">
          Welcome back
          <span className="text-gradient bg-gradient-to-r from-primary to-indigo-500">!</span>
        </h1>
        <p className="mt-2 text-muted-foreground">
          Here's an overview of your account and applications.
        </p>
      </div>

      {/* Membership Status */}
      <Card className="border-border/50 overflow-hidden">
        <div className="h-1 bg-gradient-to-r from-primary via-indigo-500 to-teal-500" />
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-indigo-500">
                <CreditCard className="h-4 w-4 text-white" />
              </div>
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
                    <span className="ml-2 text-teal-600 dark:text-teal-400 font-medium">
                      Price locked at $3/month
                    </span>
                  )}
                </p>
              </div>
              <Link to="/membership">
                <Button variant="outline" size="sm" className="border-indigo-300/30 text-indigo-600 hover:bg-indigo-500/10 dark:border-indigo-500/30 dark:text-indigo-400">
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
                <Button size="sm" className="gap-2 bg-gradient-to-r from-primary to-indigo-500 text-white border-0 shadow-md shadow-primary/20">
                  Subscribe Now <ArrowRight className="h-3.5 w-3.5" />
                </Button>
              </Link>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Applications */}
      <div>
        <div className="flex items-center gap-3 mb-4">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-indigo-500 to-teal-500">
            <AppWindow className="h-4 w-4 text-white" />
          </div>
          <h2 className="text-xl font-semibold">Your Applications</h2>
        </div>

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-indigo-500" />
          </div>
        ) : (
          <div className="grid gap-4 md:grid-cols-2">
            {applications.map((app, index) => {
              const baseDomain = import.meta.env.VITE_APP_DOMAIN || 'a8n.tools'
              const appUrl = `${app.slug}.${baseDomain}`
              const gradients = [
                'from-indigo-500 to-primary',
                'from-teal-500 to-indigo-500',
              ]
              const gradient = gradients[index % gradients.length]
              return (
                <Card key={app.id} className="border-border/50 transition-all hover:shadow-lg hover:shadow-indigo-500/5">
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
                        <Button className={`w-full bg-gradient-to-r ${gradient} text-white border-0 shadow-md shadow-indigo-500/15 hover:shadow-lg hover:shadow-indigo-500/25 transition-shadow`}>
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
