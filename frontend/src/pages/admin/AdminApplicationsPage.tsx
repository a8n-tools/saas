import { useApplications } from '@/hooks/useApplications'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Loader2, AppWindow, ExternalLink } from 'lucide-react'

export function AdminApplicationsPage() {
  const { applications, isLoading } = useApplications()

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
          Manage platform applications.
        </p>
      </div>

      <div className="grid gap-6">
        {applications.map((app) => (
          <Card key={app.id}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                    <AppWindow className="h-6 w-6 text-primary" />
                  </div>
                  <div>
                    <CardTitle>{app.name}</CardTitle>
                    <CardDescription>{app.subdomain}</CardDescription>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  {app.is_maintenance && (
                    <Badge variant="warning">Maintenance</Badge>
                  )}
                  {app.is_active ? (
                    <Badge variant="success">Active</Badge>
                  ) : (
                    <Badge variant="secondary">Inactive</Badge>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground mb-4">{app.description}</p>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-6">
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">Active</span>
                    <Switch checked={app.is_active} />
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">
                      Maintenance Mode
                    </span>
                    <Switch checked={app.is_maintenance} />
                  </div>
                </div>
                <a
                  href={`https://${app.subdomain}`}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Button variant="outline" size="sm">
                    Open
                    <ExternalLink className="ml-2 h-4 w-4" />
                  </Button>
                </a>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
