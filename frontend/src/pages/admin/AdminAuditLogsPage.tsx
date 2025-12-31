import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { formatRelativeTime } from '@/lib/utils'
import { FileText, User, CreditCard, Shield, LogIn } from 'lucide-react'

export function AdminAuditLogsPage() {
  // TODO: Fetch actual audit logs from API
  const logs = [
    {
      id: '1',
      action: 'user.login',
      resource_type: 'user',
      user_email: 'user1@example.com',
      ip_address: '192.168.1.1',
      is_admin_action: false,
      created_at: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
    },
    {
      id: '2',
      action: 'subscription.created',
      resource_type: 'subscription',
      user_email: 'user2@example.com',
      ip_address: '192.168.1.2',
      is_admin_action: false,
      created_at: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
    },
    {
      id: '3',
      action: 'user.deactivated',
      resource_type: 'user',
      user_email: 'admin@example.com',
      ip_address: '192.168.1.100',
      is_admin_action: true,
      created_at: new Date(Date.now() - 1000 * 60 * 60).toISOString(),
    },
    {
      id: '4',
      action: 'password.reset',
      resource_type: 'user',
      user_email: 'user3@example.com',
      ip_address: '192.168.1.3',
      is_admin_action: false,
      created_at: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(),
    },
  ]

  const getActionIcon = (action: string) => {
    if (action.startsWith('user.login')) return LogIn
    if (action.startsWith('subscription')) return CreditCard
    if (action.includes('admin') || action.includes('deactivate')) return Shield
    return User
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Audit Logs</h1>
        <p className="mt-2 text-muted-foreground">
          View security events and user activity.
        </p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <FileText className="h-5 w-5 text-primary" />
            <CardTitle>Recent Activity</CardTitle>
          </div>
          <CardDescription>
            All security-related events are logged here.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {logs.map((log) => {
              const Icon = getActionIcon(log.action)
              return (
                <div
                  key={log.id}
                  className="flex items-start justify-between py-4 border-b last:border-0"
                >
                  <div className="flex items-start gap-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-muted">
                      <Icon className="h-5 w-5 text-muted-foreground" />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <p className="font-medium">{log.action}</p>
                        {log.is_admin_action && (
                          <Badge variant="default" className="text-xs">
                            Admin
                          </Badge>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground">
                        {log.user_email} • {log.ip_address}
                      </p>
                    </div>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    {formatRelativeTime(log.created_at)}
                  </p>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
