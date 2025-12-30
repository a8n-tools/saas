# 09 - Admin Panel

## Overview

This document contains prompts for building the admin panel including user management, subscription management, and audit logs.

## Prerequisites
- Completed 06-08 documents
- Backend admin endpoints working

---

## Prompt 9.1: Admin Layout and Navigation

```text
Create the admin panel layout with navigation.

Create src/components/layouts/AdminLayout.tsx:
```typescript
import { Outlet, Navigate, NavLink } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';
import { LoadingScreen } from '@/components/LoadingScreen';
import { Logo } from '@/components/Logo';
import {
  LayoutDashboard,
  Users,
  CreditCard,
  Grid,
  FileText,
  Bell,
  Activity,
  ArrowLeft,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';

const adminNavItems = [
  { to: '/admin', icon: LayoutDashboard, label: 'Dashboard', end: true },
  { to: '/admin/users', icon: Users, label: 'Users' },
  { to: '/admin/subscriptions', icon: CreditCard, label: 'Subscriptions' },
  { to: '/admin/applications', icon: Grid, label: 'Applications' },
  { to: '/admin/audit-logs', icon: FileText, label: 'Audit Logs' },
  { to: '/admin/notifications', icon: Bell, label: 'Notifications' },
  { to: '/admin/health', icon: Activity, label: 'System Health' },
];

export function AdminLayout() {
  const { user, isLoading } = useAuthStore();

  if (isLoading) {
    return <LoadingScreen />;
  }

  if (!user) {
    return <Navigate to="/login" replace />;
  }

  if (user.role !== 'admin') {
    return <Navigate to="/403" replace />;
  }

  return (
    <div className="min-h-screen bg-slate-900">
      {/* Sidebar */}
      <aside className="fixed inset-y-0 left-0 w-64 bg-slate-800 border-r border-slate-700">
        <div className="p-6">
          <Logo variant="light" />
          <p className="text-xs text-slate-400 mt-1">Admin Panel</p>
        </div>

        <nav className="px-4 space-y-1">
          {adminNavItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.end}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                  isActive
                    ? 'bg-primary text-white'
                    : 'text-slate-300 hover:bg-slate-700'
                )
              }
            >
              <item.icon className="h-4 w-4" />
              {item.label}
            </NavLink>
          ))}
        </nav>

        <div className="absolute bottom-4 left-4 right-4">
          <Link to="/dashboard">
            <Button variant="outline" className="w-full" size="sm">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to Dashboard
            </Button>
          </Link>
        </div>
      </aside>

      {/* Main Content */}
      <main className="ml-64 min-h-screen bg-slate-50">
        <div className="p-8">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
```
```

---

## Prompt 9.2: Admin Dashboard Page

```text
Create the admin dashboard with key metrics.

Create src/pages/admin/AdminDashboardPage.tsx:
```typescript
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Users, CreditCard, TrendingUp, AlertTriangle } from 'lucide-react';

interface AdminStats {
  total_users: number;
  new_users_this_month: number;
  active_subscriptions: number;
  mrr: number;
  payment_failures: number;
  grace_period_users: number;
}

export function AdminDashboardPage() {
  const { data: stats, isLoading } = useQuery({
    queryKey: ['admin', 'stats'],
    queryFn: async () => {
      const response = await apiClient.get('/admin/stats');
      return response.data.data as AdminStats;
    },
  });

  const { data: notifications } = useQuery({
    queryKey: ['admin', 'notifications', 'unread'],
    queryFn: async () => {
      const response = await apiClient.get('/admin/notifications?unread=true');
      return response.data.data;
    },
  });

  const { data: recentActivity } = useQuery({
    queryKey: ['admin', 'audit-logs', 'recent'],
    queryFn: async () => {
      const response = await apiClient.get('/admin/audit-logs?limit=10');
      return response.data.data;
    },
  });

  if (isLoading) {
    return (
      <div className="space-y-6">
        <h1 className="text-2xl font-bold">Admin Dashboard</h1>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {[1, 2, 3, 4].map((i) => (
            <Skeleton key={i} className="h-32" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Admin Dashboard</h1>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Total Users"
          value={stats?.total_users || 0}
          subtitle={`+${stats?.new_users_this_month || 0} this month`}
          icon={Users}
        />
        <StatCard
          title="Active Subscriptions"
          value={stats?.active_subscriptions || 0}
          icon={CreditCard}
        />
        <StatCard
          title="MRR"
          value={`$${((stats?.mrr || 0) / 100).toFixed(2)}`}
          icon={TrendingUp}
        />
        <StatCard
          title="Payment Issues"
          value={stats?.payment_failures || 0}
          subtitle={`${stats?.grace_period_users || 0} in grace period`}
          icon={AlertTriangle}
          variant={stats?.payment_failures ? 'warning' : 'default'}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Notifications */}
        <Card>
          <CardHeader>
            <CardTitle>Unread Notifications</CardTitle>
          </CardHeader>
          <CardContent>
            {notifications?.items?.length === 0 ? (
              <p className="text-muted-foreground text-sm">No unread notifications</p>
            ) : (
              <div className="space-y-2">
                {notifications?.items?.slice(0, 5).map((n: any) => (
                  <NotificationItem key={n.id} notification={n} />
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Recent Activity */}
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {recentActivity?.items?.map((log: any) => (
                <ActivityItem key={log.id} log={log} />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function StatCard({ title, value, subtitle, icon: Icon, variant = 'default' }) {
  return (
    <Card className={variant === 'warning' ? 'border-yellow-200' : ''}>
      <CardContent className="pt-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-muted-foreground">{title}</p>
            <p className="text-2xl font-bold">{value}</p>
            {subtitle && (
              <p className="text-xs text-muted-foreground mt-1">{subtitle}</p>
            )}
          </div>
          <Icon className={cn(
            "h-8 w-8",
            variant === 'warning' ? 'text-yellow-500' : 'text-muted-foreground'
          )} />
        </div>
      </CardContent>
    </Card>
  );
}
```
```

---

## Prompt 9.3: User Management Page

```text
Create the user management page with listing and actions.

Create src/pages/admin/AdminUsersPage.tsx:
```typescript
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useAppToast } from '@/hooks/useToast';
import { MoreHorizontal, Search, UserX, UserCheck, Key, Eye } from 'lucide-react';
import { format } from 'date-fns';
import { useDebounce } from '@/hooks/useDebounce';

interface User {
  id: string;
  email: string;
  role: string;
  subscription_status: string;
  email_verified: boolean;
  created_at: string;
  last_login_at: string | null;
}

export function AdminUsersPage() {
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [page, setPage] = useState(1);
  const debouncedSearch = useDebounce(search, 300);
  const queryClient = useQueryClient();
  const { success, apiError } = useAppToast();

  const { data, isLoading } = useQuery({
    queryKey: ['admin', 'users', { search: debouncedSearch, status: statusFilter, page }],
    queryFn: async () => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: '20',
      });
      if (debouncedSearch) params.set('search', debouncedSearch);
      if (statusFilter !== 'all') params.set('subscription_status', statusFilter);

      const response = await apiClient.get(`/admin/users?${params}`);
      return response.data.data;
    },
  });

  const deactivateMutation = useMutation({
    mutationFn: async (userId: string) => {
      await apiClient.post(`/admin/users/${userId}/deactivate`);
    },
    onSuccess: () => {
      success('User deactivated');
      queryClient.invalidateQueries({ queryKey: ['admin', 'users'] });
    },
    onError: apiError,
  });

  const activateMutation = useMutation({
    mutationFn: async (userId: string) => {
      await apiClient.post(`/admin/users/${userId}/activate`);
    },
    onSuccess: () => {
      success('User activated');
      queryClient.invalidateQueries({ queryKey: ['admin', 'users'] });
    },
    onError: apiError,
  });

  const resetPasswordMutation = useMutation({
    mutationFn: async (userId: string) => {
      await apiClient.post(`/admin/users/${userId}/reset-password`);
    },
    onSuccess: () => {
      success('Password reset email sent');
    },
    onError: apiError,
  });

  const impersonateMutation = useMutation({
    mutationFn: async (userId: string) => {
      const response = await apiClient.post(`/admin/users/${userId}/impersonate`);
      return response.data.data;
    },
    onSuccess: () => {
      window.open('/dashboard', '_blank');
    },
    onError: apiError,
  });

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Users</h1>
        <p className="text-muted-foreground">Manage platform users</p>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search by email..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
        <Select value={statusFilter} onValueChange={setStatusFilter}>
          <SelectTrigger className="w-48">
            <SelectValue placeholder="Subscription status" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="active">Active</SelectItem>
            <SelectItem value="none">No Subscription</SelectItem>
            <SelectItem value="grace_period">Grace Period</SelectItem>
            <SelectItem value="canceled">Canceled</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Users Table */}
      <div className="border rounded-lg">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Email</TableHead>
              <TableHead>Role</TableHead>
              <TableHead>Subscription</TableHead>
              <TableHead>Verified</TableHead>
              <TableHead>Created</TableHead>
              <TableHead>Last Login</TableHead>
              <TableHead className="w-12"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.items?.map((user: User) => (
              <TableRow key={user.id}>
                <TableCell className="font-medium">{user.email}</TableCell>
                <TableCell>
                  <Badge variant={user.role === 'admin' ? 'default' : 'secondary'}>
                    {user.role}
                  </Badge>
                </TableCell>
                <TableCell>
                  <SubscriptionBadge status={user.subscription_status} />
                </TableCell>
                <TableCell>
                  {user.email_verified ? '✓' : '✗'}
                </TableCell>
                <TableCell>
                  {format(new Date(user.created_at), 'MMM d, yyyy')}
                </TableCell>
                <TableCell>
                  {user.last_login_at
                    ? format(new Date(user.last_login_at), 'MMM d, yyyy')
                    : 'Never'}
                </TableCell>
                <TableCell>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => impersonateMutation.mutate(user.id)}>
                        <Eye className="mr-2 h-4 w-4" />
                        Impersonate
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => resetPasswordMutation.mutate(user.id)}>
                        <Key className="mr-2 h-4 w-4" />
                        Reset Password
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem
                        onClick={() => deactivateMutation.mutate(user.id)}
                        className="text-red-600"
                      >
                        <UserX className="mr-2 h-4 w-4" />
                        Deactivate
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      {/* Pagination */}
      <Pagination
        page={page}
        totalPages={data?.total_pages || 1}
        onPageChange={setPage}
      />
    </div>
  );
}

function SubscriptionBadge({ status }: { status: string }) {
  const variants: Record<string, string> = {
    active: 'bg-green-100 text-green-800',
    none: 'bg-slate-100 text-slate-800',
    grace_period: 'bg-yellow-100 text-yellow-800',
    canceled: 'bg-red-100 text-red-800',
  };

  return (
    <span className={`px-2 py-1 rounded text-xs font-medium ${variants[status] || variants.none}`}>
      {status.replace('_', ' ')}
    </span>
  );
}
```
```

---

## Prompt 9.4: Audit Logs Page

```text
Create the audit logs page with filtering.

Create src/pages/admin/AuditLogsPage.tsx:
```typescript
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { format } from 'date-fns';
import { Search, Filter } from 'lucide-react';

interface AuditLog {
  id: string;
  actor_email: string;
  actor_role: string;
  action: string;
  resource_type: string;
  resource_id: string;
  is_admin_action: boolean;
  severity: string;
  created_at: string;
  metadata: Record<string, unknown>;
}

const actionTypes = [
  'all',
  'user_login',
  'user_logout',
  'user_registered',
  'subscription_created',
  'subscription_canceled',
  'payment_succeeded',
  'payment_failed',
  'admin_user_impersonated',
  'admin_subscription_granted',
];

const severityColors: Record<string, string> = {
  info: 'bg-blue-100 text-blue-800',
  warning: 'bg-yellow-100 text-yellow-800',
  error: 'bg-red-100 text-red-800',
  critical: 'bg-red-600 text-white',
};

export function AuditLogsPage() {
  const [action, setAction] = useState('all');
  const [adminOnly, setAdminOnly] = useState(false);
  const [page, setPage] = useState(1);

  const { data, isLoading } = useQuery({
    queryKey: ['admin', 'audit-logs', { action, adminOnly, page }],
    queryFn: async () => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: '50',
      });
      if (action !== 'all') params.set('action', action);
      if (adminOnly) params.set('admin_only', 'true');

      const response = await apiClient.get(`/admin/audit-logs?${params}`);
      return response.data.data;
    },
  });

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Audit Logs</h1>
        <p className="text-muted-foreground">Track all platform activity</p>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <Select value={action} onValueChange={setAction}>
          <SelectTrigger className="w-64">
            <SelectValue placeholder="Filter by action" />
          </SelectTrigger>
          <SelectContent>
            {actionTypes.map((type) => (
              <SelectItem key={type} value={type}>
                {type === 'all' ? 'All Actions' : type.replace(/_/g, ' ')}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Button
          variant={adminOnly ? 'default' : 'outline'}
          onClick={() => setAdminOnly(!adminOnly)}
        >
          <Filter className="mr-2 h-4 w-4" />
          Admin Actions Only
        </Button>
      </div>

      {/* Logs List */}
      <div className="space-y-2">
        {data?.items?.map((log: AuditLog) => (
          <Card key={log.id}>
            <CardContent className="py-4">
              <div className="flex items-start justify-between">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <Badge className={severityColors[log.severity]}>
                      {log.severity}
                    </Badge>
                    <span className="font-medium">
                      {log.action.replace(/_/g, ' ')}
                    </span>
                    {log.is_admin_action && (
                      <Badge variant="outline">Admin</Badge>
                    )}
                  </div>
                  <p className="text-sm text-muted-foreground">
                    by {log.actor_email || 'System'} ({log.actor_role || 'N/A'})
                  </p>
                  {log.resource_type && (
                    <p className="text-sm text-muted-foreground">
                      {log.resource_type}: {log.resource_id}
                    </p>
                  )}
                </div>
                <span className="text-sm text-muted-foreground">
                  {format(new Date(log.created_at), 'MMM d, yyyy HH:mm:ss')}
                </span>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Pagination
        page={page}
        totalPages={data?.total_pages || 1}
        onPageChange={setPage}
      />
    </div>
  );
}
```
```

---

## Prompt 9.5: Admin API Endpoints

```text
Implement the admin API endpoints on the backend.

Create src/handlers/admin.rs:

1. GET /v1/admin/stats:
   ```rust
   pub async fn get_stats(
       _user: AuthenticatedUser,  // Require admin via guard
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError> {
       // Query counts from database
       let total_users = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
           .fetch_one(&**pool)
           .await?;

       let active_subscriptions = sqlx::query_scalar!(
           "SELECT COUNT(*) FROM users WHERE subscription_status = 'active'"
       ).fetch_one(&**pool).await?;

       // Calculate MRR from active subscriptions
       let mrr = sqlx::query_scalar!(
           "SELECT COALESCE(SUM(locked_price_amount), 0) FROM users WHERE subscription_status = 'active'"
       ).fetch_one(&**pool).await?;

       // Return stats
   }
   ```

2. GET /v1/admin/users with pagination and filters

3. GET /v1/admin/users/:id with full user details

4. POST /v1/admin/users/:id/activate
5. POST /v1/admin/users/:id/deactivate
6. POST /v1/admin/users/:id/reset-password
7. POST /v1/admin/users/:id/impersonate:
   - Create audit log with admin action
   - Generate tokens for target user
   - Mark tokens as impersonation tokens
   - Return tokens

8. POST /v1/admin/users/:id/subscription/grant:
   - Grant free subscription
   - Create audit log

9. POST /v1/admin/users/:id/subscription/revoke

10. GET /v1/admin/audit-logs with filters:
    - action
    - actor_id
    - admin_only
    - date_from
    - date_to
    - severity

11. GET /v1/admin/notifications
12. POST /v1/admin/notifications/:id/read

13. GET /v1/admin/health:
    - Database status
    - Redis status
    - App container health checks
    - Memory usage
    - Active connections

All admin endpoints should:
- Require admin role
- Create audit logs for mutations
- Use proper pagination
- Return consistent response format
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Admin layout renders correctly
- [ ] Admin dashboard shows stats
- [ ] User list loads with pagination
- [ ] User search works
- [ ] User actions work (deactivate, activate)
- [ ] Password reset email sent
- [ ] Impersonation works
- [ ] Audit logs display correctly
- [ ] Audit log filters work
- [ ] Admin-only routes block non-admins

---

## Next Steps

Proceed to **[10-email-system.md](./10-email-system.md)** to implement email functionality.
