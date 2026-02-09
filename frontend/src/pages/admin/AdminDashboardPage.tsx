import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Users, CreditCard, AlertTriangle, TrendingUp, Loader2, Activity } from 'lucide-react'
import { adminApi, type AdminAuditLog } from '@/api/admin'
import { formatRelativeTime } from '@/lib/utils'

export function AdminDashboardPage() {
  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['admin', 'stats'],
    queryFn: adminApi.getStats,
  })

  const { data: recentLogs } = useQuery({
    queryKey: ['admin', 'audit-logs', 'recent'],
    queryFn: () => adminApi.getAuditLogs(1, 5),
  })

  const formatAction = (action: string) => {
    return action.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Admin Dashboard</h1>
        <p className="mt-2 text-muted-foreground">
          Overview of your platform.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Total Users</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {statsLoading ? (
              <Loader2 className="h-6 w-6 animate-spin" />
            ) : (
              <>
                <div className="text-2xl font-bold">{stats?.total_users ?? 0}</div>
                <p className="text-xs text-muted-foreground">Registered accounts</p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Active Memberships</CardTitle>
            <CreditCard className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {statsLoading ? (
              <Loader2 className="h-6 w-6 animate-spin" />
            ) : (
              <>
                <div className="text-2xl font-bold">{stats?.active_members ?? 0}</div>
                <p className="text-xs text-muted-foreground">Paying customers</p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Active Apps</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {statsLoading ? (
              <Loader2 className="h-6 w-6 animate-spin" />
            ) : (
              <>
                <div className="text-2xl font-bold">{stats?.active_applications ?? 0}/{stats?.total_applications ?? 0}</div>
                <p className="text-xs text-muted-foreground">Applications online</p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Past Due</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {statsLoading ? (
              <Loader2 className="h-6 w-6 animate-spin" />
            ) : (
              <>
                <div className="text-2xl font-bold">{stats?.past_due_members ?? 0}</div>
                <p className="text-xs text-muted-foreground">In grace period</p>
              </>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>Latest platform events</CardDescription>
          </CardHeader>
          <CardContent>
            {recentLogs?.items && recentLogs.items.length > 0 ? (
              <div className="space-y-4">
                {recentLogs.items.map((log: AdminAuditLog) => (
                  <div key={log.id} className="flex items-center gap-3">
                    <Activity className="h-4 w-4 text-muted-foreground" />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">
                        {formatAction(log.action)}
                      </p>
                      <p className="text-xs text-muted-foreground truncate">
                        {log.actor_email || 'System'}
                      </p>
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {formatRelativeTime(log.created_at)}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-muted-foreground text-center py-8">
                No recent activity
              </p>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>System Health</CardTitle>
            <CardDescription>Platform status</CardDescription>
          </CardHeader>
          <CardContent>
            <SystemHealth />
          </CardContent>
        </Card>
      </div>
    </div>
  )
}

function SystemHealth() {
  const { data: health, isLoading } = useQuery({
    queryKey: ['admin', 'health'],
    queryFn: adminApi.getHealth,
    refetchInterval: 30000,
  })

  if (isLoading) {
    return <Loader2 className="h-6 w-6 animate-spin mx-auto" />
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-sm">API Status</span>
        <span className={`text-sm font-medium ${health?.status === 'healthy' ? 'text-green-600' : 'text-red-600'}`}>
          {health?.status === 'healthy' ? 'Healthy' : 'Unhealthy'}
        </span>
      </div>
      <div className="flex items-center justify-between">
        <span className="text-sm">Database</span>
        <span className={`text-sm font-medium ${health?.database === 'connected' ? 'text-green-600' : 'text-red-600'}`}>
          {health?.database === 'connected' ? 'Connected' : 'Disconnected'}
        </span>
      </div>
      {health?.uptime_seconds && (
        <div className="flex items-center justify-between">
          <span className="text-sm">Uptime</span>
          <span className="text-sm font-medium">
            {Math.floor(health.uptime_seconds / 3600)}h {Math.floor((health.uptime_seconds % 3600) / 60)}m
          </span>
        </div>
      )}
    </div>
  )
}
