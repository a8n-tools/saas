import { useAuthStore } from '@/stores/authStore'
import { useApplications } from '@/hooks/useApplications'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ExternalLink, Loader2, Link2, Bookmark, ArrowRight } from 'lucide-react'
import { Link } from 'react-router-dom'

const appIcons: Record<string, React.ComponentType<{ className?: string }>> = {
  rus: Link2,
  rustylinks: Bookmark,
}

const appGradients: Record<string, string> = {
  rus: 'from-indigo-500 to-primary',
  rustylinks: 'from-teal-500 to-indigo-500',
}

export function ApplicationsPage() {
  const { user } = useAuthStore()
  const { applications, isLoading } = useApplications()

  const hasActiveMembership =
    user?.membership_status === 'active' ||
    user?.membership_status === 'past_due'

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-indigo-500" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Applications</h1>
        <p className="mt-2 text-muted-foreground">
          Access all your tools in one place.
        </p>
      </div>

      {!hasActiveMembership && (
        <Card className="border-indigo-500/20 bg-gradient-to-r from-indigo-500/5 via-primary/5 to-teal-500/5 overflow-hidden">
          <div className="h-1 bg-gradient-to-r from-indigo-500 via-primary to-teal-500" />
          <CardContent className="flex items-center justify-between py-4">
            <div>
              <p className="font-medium">Membership required</p>
              <p className="text-sm text-muted-foreground">
                Subscribe to access all applications.
              </p>
            </div>
            <Link to="/membership">
              <Button className="gap-2 bg-gradient-to-r from-primary to-indigo-500 text-white border-0 shadow-md shadow-primary/20">
                Subscribe Now <ArrowRight className="h-3.5 w-3.5" />
              </Button>
            </Link>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {applications.map((app) => {
          const Icon = appIcons[app.slug] || Link2
          const gradient = appGradients[app.slug] || 'from-primary to-indigo-500'
          const baseDomain = import.meta.env.VITE_APP_DOMAIN || 'a8n.tools'
          const appUrl = `${app.slug}.${baseDomain}`

          return (
            <Card
              key={app.id}
              className={`border-border/50 transition-all hover:shadow-lg hover:shadow-indigo-500/5 ${!app.is_accessible ? 'opacity-75' : ''}`}
            >
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className={`flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-br ${gradient}`}>
                    <Icon className="h-6 w-6 text-white" />
                  </div>
                  <div className="flex gap-2">
                    {app.maintenance_mode && (
                      <Badge variant="warning">Maintenance</Badge>
                    )}
                  </div>
                </div>
                <CardTitle className="mt-4">{app.display_name}</CardTitle>
                <CardDescription>{app.description}</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-4">
                  {appUrl}
                </p>
                {app.is_accessible ? (
                  <a
                    href={`https://${appUrl}`}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <Button className={`w-full bg-gradient-to-r ${gradient} text-white border-0 shadow-md shadow-indigo-500/15 hover:shadow-lg hover:shadow-indigo-500/25 transition-shadow`}>
                      Launch
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
    </div>
  )
}
