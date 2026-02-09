import { useAuthStore } from '@/stores/authStore'
import { useApplications } from '@/hooks/useApplications'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ExternalLink, Loader2, Link2, Bookmark } from 'lucide-react'
import { Link } from 'react-router-dom'

const appIcons: Record<string, React.ComponentType<{ className?: string }>> = {
  rus: Link2,
  rustylinks: Bookmark,
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
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Applications</h1>
        <p className="mt-2 text-muted-foreground">
          Access all your developer tools in one place.
        </p>
      </div>

      {!hasActiveMembership && (
        <Card className="border-primary bg-primary/5">
          <CardContent className="flex items-center justify-between py-4">
            <div>
              <p className="font-medium">Membership required</p>
              <p className="text-sm text-muted-foreground">
                Subscribe to access all applications.
              </p>
            </div>
            <Link to="/membership">
              <Button>Subscribe Now</Button>
            </Link>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {applications.map((app) => {
          const Icon = appIcons[app.slug] || Link2
          const isMaintenance = app.maintenance_mode ?? app.is_maintenance ?? false
          const appName = app.display_name || app.name
          const appUrl = app.subdomain || `${app.slug}.a8n.tools`
          const canAccess = hasActiveMembership && app.is_active && !isMaintenance

          return (
            <Card
              key={app.id}
              className={!canAccess ? 'opacity-75' : ''}
            >
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                    <Icon className="h-6 w-6 text-primary" />
                  </div>
                  <div className="flex gap-2">
                    {isMaintenance && (
                      <Badge variant="warning">Maintenance</Badge>
                    )}
                    {!app.is_active && (
                      <Badge variant="secondary">Inactive</Badge>
                    )}
                  </div>
                </div>
                <CardTitle className="mt-4">{appName}</CardTitle>
                <CardDescription>{app.description}</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-4">
                  {appUrl}
                </p>
                {canAccess ? (
                  <a
                    href={`https://${appUrl}`}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <Button className="w-full">
                      Launch
                      <ExternalLink className="ml-2 h-4 w-4" />
                    </Button>
                  </a>
                ) : (
                  <Button className="w-full" disabled>
                    {!hasActiveMembership
                      ? 'Membership Required'
                      : isMaintenance
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
