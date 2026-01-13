import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { formatRelativeTime } from '@/lib/utils'
import { FileText, User, CreditCard, Shield, LogIn, Loader2, KeyRound, Mail, UserPlus } from 'lucide-react'
import { adminApi, type AdminAuditLog } from '@/api/admin'

export function AdminAuditLogsPage() {
  const [page, setPage] = useState(1)
  const [adminOnly, setAdminOnly] = useState(false)

  const { data, isLoading } = useQuery({
    queryKey: ['admin', 'audit-logs', page, adminOnly],
    queryFn: () => adminApi.getAuditLogs(page, 50, { admin_only: adminOnly }),
  })

  const getActionIcon = (action: string) => {
    if (action.includes('login')) return LogIn
    if (action.includes('subscription') || action.includes('payment')) return CreditCard
    if (action.includes('password')) return KeyRound
    if (action.includes('magic_link')) return Mail
    if (action.includes('register')) return UserPlus
    if (action.includes('admin') || action.includes('deactivate')) return Shield
    return User
  }

  const formatAction = (action: string) => {
    return action.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
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
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <FileText className="h-5 w-5 text-primary" />
              <div>
                <CardTitle>Recent Activity</CardTitle>
                <CardDescription>
                  All security-related events are logged here.
                </CardDescription>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Admin actions only</span>
              <Switch
                checked={adminOnly}
                onCheckedChange={(checked) => {
                  setAdminOnly(checked)
                  setPage(1)
                }}
              />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin" />
            </div>
          ) : (
            <>
              <div className="space-y-4">
                {data?.items.map((log: AdminAuditLog) => {
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
                            <p className="font-medium">{formatAction(log.action)}</p>
                            {log.is_admin_action && (
                              <Badge variant="default" className="text-xs">
                                Admin
                              </Badge>
                            )}
                            {log.severity === 'warning' && (
                              <Badge variant="warning" className="text-xs">
                                Warning
                              </Badge>
                            )}
                            {log.severity === 'error' && (
                              <Badge variant="destructive" className="text-xs">
                                Error
                              </Badge>
                            )}
                          </div>
                          <p className="text-sm text-muted-foreground">
                            {log.actor_email || 'System'}
                            {log.actor_ip_address && ` • ${log.actor_ip_address}`}
                          </p>
                          {log.resource_type && (
                            <p className="text-xs text-muted-foreground mt-1">
                              Resource: {log.resource_type}
                              {log.resource_id && ` (${log.resource_id.slice(0, 8)}...)`}
                            </p>
                          )}
                        </div>
                      </div>
                      <p className="text-sm text-muted-foreground whitespace-nowrap">
                        {formatRelativeTime(log.created_at)}
                      </p>
                    </div>
                  )
                })}
                {data?.items.length === 0 && (
                  <p className="text-center text-muted-foreground py-8">
                    No audit logs found
                  </p>
                )}
              </div>

              {data && data.total_pages > 1 && (
                <div className="flex justify-center gap-2 mt-6">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage((p) => Math.max(1, p - 1))}
                    disabled={page === 1}
                  >
                    Previous
                  </Button>
                  <span className="flex items-center px-3 text-sm">
                    Page {page} of {data.total_pages}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage((p) => Math.min(data.total_pages, p + 1))}
                    disabled={page === data.total_pages}
                  >
                    Next
                  </Button>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
