import { useMemo } from 'react'
import { Link } from 'react-router-dom'
import { config } from '@/config'
import { useAuthStore } from '@/stores/authStore'
import { useApplications } from '@/hooks/useApplications'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { AppWindow, CreditCard, ExternalLink, Loader2, ArrowRight } from 'lucide-react'
import { getAppGradient, hasActiveMembership } from '@/lib/utils'

const taglines = [
  'All access. No clock.',
  'All in. All yours.',
  'No teardown. No sunset. Just tools.',
  'Price locked. Tools stocked.',
  'Zero TTL. Infinite access.',
]

export function DashboardPage() {
  const { user } = useAuthStore()
  const { applications, isLoading } = useApplications()
  const tagline = useMemo(() => taglines[Math.floor(Math.random() * taglines.length)], [])

  const isMember = hasActiveMembership(user)

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">
          Welcome back!
        </h1>
        <p className="mt-2 text-muted-foreground">
          {tagline}
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
            <MembershipBadge user={user} />
          </div>
        </CardHeader>
        <CardContent>
          {isMember ? (
            <div className="flex items-center justify-between">
              <div>
                <SubscriptionStatusText user={user} />
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
                {user?.trial_ends_at || user?.membership_status === 'canceled'
                  ? 'Your trial has ended — subscribe to continue.'
                  : 'Subscribe to get access to all applications.'}
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
              const baseDomain = config.appDomain || 'localhost'
              const subdomain = app.subdomain || app.slug
              const appUrl = `${subdomain}.${baseDomain}`
              const gradient = getAppGradient(index)
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
                        {!isMember
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

function MembershipBadge({ user }: { user?: import('@/types').User | null }) {
  if (user?.lifetime_member) return <Badge variant="success">Lifetime</Badge>
  if (user?.trial_ends_at && new Date(user.trial_ends_at) > new Date()) {
    return <Badge variant="secondary">Trial</Badge>
  }
  switch (user?.membership_status) {
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

function SubscriptionStatusText({ user }: { user?: import('@/types').User | null }) {
  if (user?.lifetime_member) {
    return <p className="text-sm font-medium text-teal-600 dark:text-teal-400">Lifetime member 🎉</p>
  }
  if (user?.trial_ends_at) {
    const trialEnd = new Date(user.trial_ends_at)
    const now = new Date()
    if (trialEnd > now) {
      const daysLeft = Math.ceil((trialEnd.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
      return <p className="text-sm text-muted-foreground">Trial ends in {daysLeft} day{daysLeft !== 1 ? 's' : ''}</p>
    }
  }
  if (user?.membership_status === 'active') {
    return (
      <p className="text-sm text-muted-foreground">
        You have access to all applications.
        {user.price_locked && (
          <span className="ml-2 text-teal-600 dark:text-teal-400 font-medium">
            Price locked at $3/month
          </span>
        )}
      </p>
    )
  }
  return <p className="text-sm text-muted-foreground">You have access to all applications.</p>
}
