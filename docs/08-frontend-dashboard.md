# 08 - Frontend Dashboard & Subscription

## Overview

This document contains prompts for building the user dashboard, application cards, and subscription management UI.

## Prerequisites
- Completed 06-07 documents
- Backend subscription endpoints working

---

## Prompt 8.1: Dashboard Layout and Navigation

```text
Create the main dashboard layout with navigation.

Create src/components/DashboardNav.tsx:
```typescript
import { NavLink } from 'react-router-dom';
import { Home, Grid, User, CreditCard, Settings } from 'lucide-react';
import { cn } from '@/lib/utils';

const navItems = [
  { to: '/dashboard', icon: Home, label: 'Overview', end: true },
  { to: '/dashboard/apps', icon: Grid, label: 'Applications' },
  { to: '/dashboard/account', icon: User, label: 'Account' },
  { to: '/dashboard/subscription', icon: CreditCard, label: 'Subscription' },
];

export function DashboardNav() {
  return (
    <aside className="w-64 border-r bg-white min-h-[calc(100vh-64px)]">
      <nav className="p-4 space-y-1">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.end}
            className={({ isActive }) =>
              cn(
                'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                isActive
                  ? 'bg-primary/10 text-primary'
                  : 'text-slate-600 hover:bg-slate-100'
              )
            }
          >
            <item.icon className="h-4 w-4" />
            {item.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
```

Create src/components/DashboardHeader.tsx:
```typescript
import { Link } from 'react-router-dom';
import { useAuth } from '@/hooks/useAuth';
import { Logo } from './Logo';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { LogOut, Settings, User as UserIcon } from 'lucide-react';

export function DashboardHeader() {
  const { user, logout } = useAuth();

  const initials = user?.email
    .split('@')[0]
    .slice(0, 2)
    .toUpperCase();

  return (
    <header className="h-16 border-b bg-white flex items-center justify-between px-6">
      <Link to="/dashboard">
        <Logo />
      </Link>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="flex items-center gap-2">
            <Avatar className="h-8 w-8">
              <AvatarFallback className="bg-primary text-white text-sm">
                {initials}
              </AvatarFallback>
            </Avatar>
            <span className="text-sm font-medium hidden sm:block">
              {user?.email}
            </span>
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-48">
          <DropdownMenuItem asChild>
            <Link to="/dashboard/account" className="cursor-pointer">
              <UserIcon className="mr-2 h-4 w-4" />
              Account
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link to="/dashboard/subscription" className="cursor-pointer">
              <Settings className="mr-2 h-4 w-4" />
              Subscription
            </Link>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => logout()}
            className="text-red-600 cursor-pointer"
          >
            <LogOut className="mr-2 h-4 w-4" />
            Sign out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </header>
  );
}
```
```

---

## Prompt 8.2: Dashboard Overview Page

```text
Create the main dashboard overview page.

Create src/pages/dashboard/DashboardPage.tsx:
```typescript
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { apiClient } from '@/api/client';
import { useAuthStore } from '@/stores/authStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { Grid, CreditCard, AlertCircle, CheckCircle } from 'lucide-react';

export function DashboardPage() {
  const { user } = useAuthStore();

  const { data: applications, isLoading: appsLoading } = useQuery({
    queryKey: ['applications'],
    queryFn: async () => {
      const response = await apiClient.get('/applications');
      return response.data.data.applications;
    },
  });

  const { data: subscription, isLoading: subLoading } = useQuery({
    queryKey: ['subscription'],
    queryFn: async () => {
      const response = await apiClient.get('/subscriptions/me');
      return response.data.data;
    },
  });

  const hasActiveSubscription = ['active', 'grace_period'].includes(
    user?.subscription_status || ''
  );

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Welcome back!</h1>
        <p className="text-muted-foreground">
          Here's an overview of your example.com account
        </p>
      </div>

      {/* Subscription Status Banner */}
      {user?.subscription_status === 'grace_period' && (
        <Card className="border-yellow-200 bg-yellow-50">
          <CardContent className="flex items-center gap-4 py-4">
            <AlertCircle className="h-5 w-5 text-yellow-600" />
            <div className="flex-1">
              <p className="font-medium text-yellow-800">
                Payment issue detected
              </p>
              <p className="text-sm text-yellow-700">
                Please update your payment method to continue access.
              </p>
            </div>
            <Link to="/dashboard/subscription">
              <Button variant="outline" size="sm">
                Update Payment
              </Button>
            </Link>
          </CardContent>
        </Card>
      )}

      {!hasActiveSubscription && (
        <Card className="border-primary/20 bg-primary/5">
          <CardContent className="flex items-center gap-4 py-4">
            <CreditCard className="h-5 w-5 text-primary" />
            <div className="flex-1">
              <p className="font-medium">Unlock all applications</p>
              <p className="text-sm text-muted-foreground">
                Subscribe for $3/month and get access to all tools.
              </p>
            </div>
            <Link to="/pricing">
              <Button size="sm">Subscribe Now</Button>
            </Link>
          </CardContent>
        </Card>
      )}

      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Subscription Status</CardDescription>
            <CardTitle className="flex items-center gap-2">
              {hasActiveSubscription ? (
                <>
                  <CheckCircle className="h-5 w-5 text-green-500" />
                  Active
                </>
              ) : (
                <>
                  <AlertCircle className="h-5 w-5 text-slate-400" />
                  Inactive
                </>
              )}
            </CardTitle>
          </CardHeader>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Available Applications</CardDescription>
            <CardTitle>{applications?.length || 0}</CardTitle>
          </CardHeader>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Monthly Price</CardDescription>
            <CardTitle>
              {user?.price_locked ? (
                <span>$3/mo <Badge variant="secondary">Locked</Badge></span>
              ) : (
                '$3/mo'
              )}
            </CardTitle>
          </CardHeader>
        </Card>
      </div>

      {/* Applications Preview */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Your Applications</CardTitle>
              <CardDescription>
                Quick access to your tools
              </CardDescription>
            </div>
            <Link to="/dashboard/apps">
              <Button variant="outline" size="sm">
                View All
              </Button>
            </Link>
          </div>
        </CardHeader>
        <CardContent>
          {appsLoading ? (
            <div className="grid grid-cols-2 gap-4">
              <Skeleton className="h-24" />
              <Skeleton className="h-24" />
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              {applications?.slice(0, 2).map((app: any) => (
                <AppPreviewCard
                  key={app.id}
                  app={app}
                  hasAccess={hasActiveSubscription}
                />
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function AppPreviewCard({ app, hasAccess }: { app: any; hasAccess: boolean }) {
  return (
    <div className="border rounded-lg p-4 flex items-center gap-4">
      <div className="h-12 w-12 rounded-lg bg-primary/10 flex items-center justify-center">
        <Grid className="h-6 w-6 text-primary" />
      </div>
      <div className="flex-1">
        <h3 className="font-medium">{app.display_name}</h3>
        <p className="text-sm text-muted-foreground line-clamp-1">
          {app.description}
        </p>
      </div>
      {hasAccess ? (
        <a
          href={`https://${app.slug}.example.com`}
          target="_blank"
          rel="noopener noreferrer"
        >
          <Button size="sm">Open</Button>
        </a>
      ) : (
        <Button size="sm" variant="outline" disabled>
          Locked
        </Button>
      )}
    </div>
  );
}
```
```

---

## Prompt 8.3: Applications Page

```text
Create the applications listing page with app cards.

Create src/pages/dashboard/AppsPage.tsx:
```typescript
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { useAuthStore } from '@/stores/authStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { ExternalLink, Lock, Github, Wrench } from 'lucide-react';

interface Application {
  id: string;
  slug: string;
  display_name: string;
  description: string;
  icon_url: string | null;
  version: string;
  source_code_url: string | null;
  maintenance_mode: boolean;
  maintenance_message: string | null;
}

export function AppsPage() {
  const { user } = useAuthStore();

  const { data: applications, isLoading } = useQuery({
    queryKey: ['applications'],
    queryFn: async () => {
      const response = await apiClient.get('/applications');
      return response.data.data.applications as Application[];
    },
  });

  const hasActiveSubscription = ['active', 'grace_period'].includes(
    user?.subscription_status || ''
  );

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold">Applications</h1>
          <p className="text-muted-foreground">
            Access your developer tools
          </p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {[1, 2].map((i) => (
            <Skeleton key={i} className="h-64" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Applications</h1>
        <p className="text-muted-foreground">
          {hasActiveSubscription
            ? 'Access all your developer tools'
            : 'Subscribe to unlock all applications'}
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {applications?.map((app) => (
          <ApplicationCard
            key={app.id}
            app={app}
            hasAccess={hasActiveSubscription}
          />
        ))}
      </div>
    </div>
  );
}

function ApplicationCard({
  app,
  hasAccess,
}: {
  app: Application;
  hasAccess: boolean;
}) {
  const appUrl = `https://${app.slug}.example.com`;

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="h-12 w-12 rounded-lg bg-gradient-to-br from-primary to-rust flex items-center justify-center text-white font-bold text-lg">
            {app.display_name[0]}
          </div>
          {app.maintenance_mode && (
            <Badge variant="secondary" className="flex items-center gap-1">
              <Wrench className="h-3 w-3" />
              Maintenance
            </Badge>
          )}
        </div>
        <CardTitle className="mt-4">{app.display_name}</CardTitle>
        <CardDescription className="line-clamp-2">
          {app.description}
        </CardDescription>
      </CardHeader>

      <CardContent className="flex-1 flex flex-col justify-end space-y-4">
        <div className="flex items-center justify-between text-sm text-muted-foreground">
          <span>Version {app.version}</span>
          {app.source_code_url && (
            <a
              href={app.source_code_url}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1 hover:text-foreground"
            >
              <Github className="h-4 w-4" />
              Source
            </a>
          )}
        </div>

        {app.maintenance_mode ? (
          <div className="p-3 bg-yellow-50 rounded-lg text-sm text-yellow-800">
            {app.maintenance_message || 'This application is under maintenance.'}
          </div>
        ) : hasAccess ? (
          <a href={appUrl} target="_blank" rel="noopener noreferrer">
            <Button className="w-full">
              <ExternalLink className="mr-2 h-4 w-4" />
              Open Application
            </Button>
          </a>
        ) : (
          <Button variant="outline" disabled className="w-full">
            <Lock className="mr-2 h-4 w-4" />
            Subscription Required
          </Button>
        )}
      </CardContent>
    </Card>
  );
}
```
```

---

## Prompt 8.4: Subscription Management Page

```text
Create the subscription management page.

Create src/pages/dashboard/SubscriptionPage.tsx:
```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { useAuthStore } from '@/stores/authStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { useAppToast } from '@/hooks/useToast';
import { CheckCircle, CreditCard, AlertTriangle, ExternalLink } from 'lucide-react';
import { format } from 'date-fns';

interface SubscriptionData {
  status: string;
  price_locked: boolean;
  locked_price_amount: number | null;
  current_period_end: string | null;
  cancel_at_period_end: boolean;
  grace_period_end: string | null;
}

export function SubscriptionPage() {
  const queryClient = useQueryClient();
  const { user } = useAuthStore();
  const { success, apiError } = useAppToast();

  const { data: subscription, isLoading } = useQuery({
    queryKey: ['subscription'],
    queryFn: async () => {
      const response = await apiClient.get('/subscriptions/me');
      return response.data.data as SubscriptionData;
    },
  });

  const cancelMutation = useMutation({
    mutationFn: async () => {
      await apiClient.post('/subscriptions/cancel');
    },
    onSuccess: () => {
      success('Subscription canceled', 'You will retain access until the end of your billing period.');
      queryClient.invalidateQueries({ queryKey: ['subscription'] });
    },
    onError: apiError,
  });

  const reactivateMutation = useMutation({
    mutationFn: async () => {
      await apiClient.post('/subscriptions/reactivate');
    },
    onSuccess: () => {
      success('Subscription reactivated!');
      queryClient.invalidateQueries({ queryKey: ['subscription'] });
    },
    onError: apiError,
  });

  const portalMutation = useMutation({
    mutationFn: async () => {
      const response = await apiClient.post('/subscriptions/billing-portal');
      return response.data.data.url;
    },
    onSuccess: (url) => {
      window.open(url, '_blank');
    },
    onError: apiError,
  });

  const checkoutMutation = useMutation({
    mutationFn: async () => {
      const response = await apiClient.post('/subscriptions/checkout');
      return response.data.data;
    },
    onSuccess: (data) => {
      window.location.href = data.checkout_url;
    },
    onError: apiError,
  });

  const hasSubscription = ['active', 'past_due', 'grace_period'].includes(
    subscription?.status || ''
  );

  if (isLoading) {
    return <div>Loading subscription...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Subscription</h1>
        <p className="text-muted-foreground">
          Manage your example.com subscription
        </p>
      </div>

      {/* Current Plan */}
      <Card>
        <CardHeader>
          <CardTitle>Current Plan</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-slate-50 rounded-lg">
            <div>
              <p className="font-medium">example.com Subscription</p>
              <p className="text-2xl font-bold text-primary">
                ${subscription?.locked_price_amount ? subscription.locked_price_amount / 100 : 3}/month
              </p>
              {subscription?.price_locked && (
                <Badge variant="secondary" className="mt-1">
                  Price locked forever
                </Badge>
              )}
            </div>
            <StatusBadge status={subscription?.status || 'none'} />
          </div>

          {subscription?.current_period_end && (
            <p className="text-sm text-muted-foreground">
              {subscription.cancel_at_period_end
                ? `Access until ${format(new Date(subscription.current_period_end), 'MMMM d, yyyy')}`
                : `Next billing date: ${format(new Date(subscription.current_period_end), 'MMMM d, yyyy')}`}
            </p>
          )}

          {subscription?.grace_period_end && (
            <div className="p-4 bg-yellow-50 rounded-lg border border-yellow-200">
              <div className="flex items-start gap-3">
                <AlertTriangle className="h-5 w-5 text-yellow-600 mt-0.5" />
                <div>
                  <p className="font-medium text-yellow-800">Payment Required</p>
                  <p className="text-sm text-yellow-700">
                    Please update your payment method before{' '}
                    {format(new Date(subscription.grace_period_end), 'MMMM d, yyyy')}{' '}
                    to maintain access.
                  </p>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Actions */}
      <Card>
        <CardHeader>
          <CardTitle>Manage Subscription</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {!hasSubscription && (
            <Button onClick={() => checkoutMutation.mutate()} disabled={checkoutMutation.isPending}>
              <CreditCard className="mr-2 h-4 w-4" />
              {checkoutMutation.isPending ? 'Loading...' : 'Subscribe Now'}
            </Button>
          )}

          {hasSubscription && (
            <>
              <Button
                variant="outline"
                onClick={() => portalMutation.mutate()}
                disabled={portalMutation.isPending}
              >
                <ExternalLink className="mr-2 h-4 w-4" />
                Manage Payment Method
              </Button>

              {subscription?.cancel_at_period_end ? (
                <Button
                  variant="outline"
                  onClick={() => reactivateMutation.mutate()}
                  disabled={reactivateMutation.isPending}
                >
                  {reactivateMutation.isPending ? 'Reactivating...' : 'Reactivate Subscription'}
                </Button>
              ) : (
                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <Button variant="outline" className="text-red-600">
                      Cancel Subscription
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Cancel subscription?</AlertDialogTitle>
                      <AlertDialogDescription>
                        You'll retain access until the end of your current billing period.
                        You can reactivate anytime before then.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Keep Subscription</AlertDialogCancel>
                      <AlertDialogAction
                        onClick={() => cancelMutation.mutate()}
                        className="bg-red-600 hover:bg-red-700"
                      >
                        {cancelMutation.isPending ? 'Canceling...' : 'Yes, Cancel'}
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              )}
            </>
          )}
        </CardContent>
      </Card>

      {/* Benefits */}
      <Card>
        <CardHeader>
          <CardTitle>What's Included</CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="space-y-3">
            {[
              'Access to all current applications',
              'Access to all future applications',
              'Priority support',
              'Price locked at signup rate forever',
            ].map((benefit) => (
              <li key={benefit} className="flex items-center gap-2">
                <CheckCircle className="h-4 w-4 text-green-500" />
                <span>{benefit}</span>
              </li>
            ))}
          </ul>
        </CardContent>
      </Card>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const variants: Record<string, { className: string; label: string }> = {
    active: { className: 'bg-green-100 text-green-800', label: 'Active' },
    grace_period: { className: 'bg-yellow-100 text-yellow-800', label: 'Grace Period' },
    past_due: { className: 'bg-red-100 text-red-800', label: 'Past Due' },
    canceled: { className: 'bg-slate-100 text-slate-800', label: 'Canceled' },
    none: { className: 'bg-slate-100 text-slate-800', label: 'No Subscription' },
  };

  const variant = variants[status] || variants.none;

  return (
    <span className={`px-3 py-1 rounded-full text-sm font-medium ${variant.className}`}>
      {variant.label}
    </span>
  );
}
```
```

---

## Prompt 8.5: Account Settings Page

```text
Create the account settings page.

Create src/pages/dashboard/AccountPage.tsx:
```typescript
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import { z } from 'zod';
import { apiClient } from '@/api/client';
import { useAuthStore } from '@/stores/authStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { useAppToast } from '@/hooks/useToast';
import { PasswordStrengthIndicator } from '@/components/PasswordStrengthIndicator';
import { Link } from 'react-router-dom';
import { CheckCircle, Mail, Shield } from 'lucide-react';

const changePasswordSchema = z.object({
  current_password: z.string().min(1, 'Current password is required'),
  new_password: z.string().min(12, 'Password must be at least 12 characters'),
  confirm_password: z.string(),
}).refine((data) => data.new_password === data.confirm_password, {
  message: 'Passwords do not match',
  path: ['confirm_password'],
});

type ChangePasswordInput = z.infer<typeof changePasswordSchema>;

export function AccountPage() {
  const { user } = useAuthStore();
  const { success, apiError } = useAppToast();

  const form = useForm<ChangePasswordInput>({
    resolver: zodResolver(changePasswordSchema),
    defaultValues: {
      current_password: '',
      new_password: '',
      confirm_password: '',
    },
  });

  const changePasswordMutation = useMutation({
    mutationFn: async (data: ChangePasswordInput) => {
      await apiClient.put('/users/me/password', {
        current_password: data.current_password,
        new_password: data.new_password,
      });
    },
    onSuccess: () => {
      success('Password updated successfully');
      form.reset();
    },
    onError: apiError,
  });

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Account Settings</h1>
        <p className="text-muted-foreground">
          Manage your account information and security
        </p>
      </div>

      {/* Profile Info */}
      <Card>
        <CardHeader>
          <CardTitle>Profile</CardTitle>
          <CardDescription>Your account information</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Mail className="h-5 w-5 text-muted-foreground" />
              <div>
                <p className="font-medium">{user?.email}</p>
                <p className="text-sm text-muted-foreground">Email address</p>
              </div>
            </div>
            {user?.email_verified ? (
              <Badge variant="secondary" className="flex items-center gap-1">
                <CheckCircle className="h-3 w-3" />
                Verified
              </Badge>
            ) : (
              <Button variant="outline" size="sm">
                Verify Email
              </Button>
            )}
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Shield className="h-5 w-5 text-muted-foreground" />
              <div>
                <p className="font-medium">Role</p>
                <p className="text-sm text-muted-foreground capitalize">
                  {user?.role}
                </p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Change Password */}
      <Card>
        <CardHeader>
          <CardTitle>Change Password</CardTitle>
          <CardDescription>
            Update your password to keep your account secure
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form
            onSubmit={form.handleSubmit((data) => changePasswordMutation.mutate(data))}
            className="space-y-4 max-w-md"
          >
            <div className="space-y-2">
              <Label htmlFor="current_password">Current Password</Label>
              <Input
                id="current_password"
                type="password"
                {...form.register('current_password')}
              />
              {form.formState.errors.current_password && (
                <p className="text-sm text-destructive">
                  {form.formState.errors.current_password.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="new_password">New Password</Label>
              <Input
                id="new_password"
                type="password"
                {...form.register('new_password')}
              />
              <PasswordStrengthIndicator password={form.watch('new_password')} />
              {form.formState.errors.new_password && (
                <p className="text-sm text-destructive">
                  {form.formState.errors.new_password.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="confirm_password">Confirm New Password</Label>
              <Input
                id="confirm_password"
                type="password"
                {...form.register('confirm_password')}
              />
              {form.formState.errors.confirm_password && (
                <p className="text-sm text-destructive">
                  {form.formState.errors.confirm_password.message}
                </p>
              )}
            </div>

            <Button
              type="submit"
              disabled={changePasswordMutation.isPending}
            >
              {changePasswordMutation.isPending ? 'Updating...' : 'Update Password'}
            </Button>
          </form>
        </CardContent>
      </Card>

      {/* Active Sessions */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Active Sessions</CardTitle>
              <CardDescription>
                Devices where you're logged in
              </CardDescription>
            </div>
            <Link to="/dashboard/sessions">
              <Button variant="outline" size="sm">
                Manage Sessions
              </Button>
            </Link>
          </div>
        </CardHeader>
      </Card>
    </div>
  );
}
```
```

---

## Prompt 8.6: Landing Page and Pricing

```text
Create the marketing landing page and pricing page.

Create src/pages/HomePage.tsx:
```typescript
import { Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Logo } from '@/components/Logo';
import { CheckCircle, Zap, Shield, Heart } from 'lucide-react';

export function HomePage() {
  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-50 to-white">
      {/* Header */}
      <header className="container mx-auto px-4 py-6 flex items-center justify-between">
        <Logo />
        <nav className="flex items-center gap-4">
          <Link to="/pricing" className="text-sm font-medium hover:text-primary">
            Pricing
          </Link>
          <Link to="/login">
            <Button variant="ghost">Sign in</Button>
          </Link>
          <Link to="/register">
            <Button>Get Started</Button>
          </Link>
        </nav>
      </header>

      {/* Hero */}
      <section className="container mx-auto px-4 py-20 text-center">
        <h1 className="text-5xl font-bold mb-6">
          Developer tools,{' '}
          <span className="text-primary">automated.</span>
        </h1>
        <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-8">
          Open source tools, managed for you. No server setup, no maintenance.
          Just the tools you need, ready when you are.
        </p>
        <div className="flex items-center justify-center gap-4">
          <Link to="/register">
            <Button size="lg">
              Start for $3/month
            </Button>
          </Link>
          <Link to="/pricing">
            <Button size="lg" variant="outline">
              View Pricing
            </Button>
          </Link>
        </div>
      </section>

      {/* Features */}
      <section className="container mx-auto px-4 py-20">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <FeatureCard
            icon={Zap}
            title="Instant Access"
            description="No setup required. Sign up and start using your tools immediately."
          />
          <FeatureCard
            icon={Shield}
            title="Always Updated"
            description="We handle updates, security patches, and maintenance."
          />
          <FeatureCard
            icon={Heart}
            title="Price for Life"
            description="Lock in your price forever. Early adopters get the best deal."
          />
        </div>
      </section>

      {/* Pricing Preview */}
      <section className="container mx-auto px-4 py-20">
        <div className="max-w-md mx-auto bg-white rounded-2xl shadow-xl p-8 border">
          <div className="text-center mb-6">
            <p className="text-sm text-muted-foreground mb-2">All-Access Subscription</p>
            <p className="text-5xl font-bold">$3<span className="text-lg font-normal">/mo</span></p>
            <p className="text-sm text-muted-foreground mt-2">Locked at signup forever</p>
          </div>
          <ul className="space-y-3 mb-8">
            {[
              'All current applications',
              'All future applications',
              'Priority support',
              'No usage limits',
            ].map((feature) => (
              <li key={feature} className="flex items-center gap-2">
                <CheckCircle className="h-5 w-5 text-green-500" />
                <span>{feature}</span>
              </li>
            ))}
          </ul>
          <Link to="/register">
            <Button className="w-full" size="lg">
              Get Started
            </Button>
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="container mx-auto px-4 py-8 border-t">
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            © 2024 example.com. All rights reserved.
          </p>
          <nav className="flex items-center gap-4 text-sm">
            <Link to="/terms" className="hover:underline">Terms</Link>
            <Link to="/privacy" className="hover:underline">Privacy</Link>
          </nav>
        </div>
      </footer>
    </div>
  );
}

function FeatureCard({
  icon: Icon,
  title,
  description,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  description: string;
}) {
  return (
    <div className="text-center p-6">
      <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary/10 mb-4">
        <Icon className="h-6 w-6 text-primary" />
      </div>
      <h3 className="text-lg font-semibold mb-2">{title}</h3>
      <p className="text-muted-foreground">{description}</p>
    </div>
  );
}
```

Create src/pages/PricingPage.tsx with more detailed pricing info.
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Dashboard navigation works correctly
- [ ] Dashboard shows subscription status
- [ ] Application cards display correctly
- [ ] Subscription page shows current plan
- [ ] Cancel subscription flow works
- [ ] Reactivate subscription works
- [ ] Billing portal redirect works
- [ ] Account page shows user info
- [ ] Password change works
- [ ] Landing page renders correctly
- [ ] All links navigate correctly

---

## Next Steps

Proceed to **[09-admin-panel.md](./09-admin-panel.md)** to build the admin interface.
