# 07 - Frontend Authentication UI

## Overview

This document contains prompts for implementing the authentication user interface including login, registration, magic links, and password reset.

## Prerequisites
- Completed 06-frontend-foundation.md
- Backend authentication endpoints working

---

## Prompt 7.1: Login Page

```text
Create the login page with email/password form.

Create src/pages/auth/LoginPage.tsx:
```typescript
import { useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { useAuth } from '@/hooks/useAuth';
import { useAppToast } from '@/hooks/useToast';
import { loginSchema, LoginInput } from '@/lib/validations/auth';

export function LoginPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { login, isLoggingIn } = useAuth();
  const { error: showError } = useAppToast();

  const returnTo = searchParams.get('returnTo') || '/dashboard';

  const form = useForm<LoginInput>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: '',
      password: '',
      remember: false,
    },
  });

  const onSubmit = async (data: LoginInput) => {
    try {
      await login(data);
      navigate(returnTo);
    } catch (err) {
      showError('Login failed', 'Invalid email or password');
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Welcome back</CardTitle>
        <CardDescription>
          Sign in to your account to continue
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              placeholder="you@example.com"
              {...form.register('email')}
            />
            {form.formState.errors.email && (
              <p className="text-sm text-destructive">
                {form.formState.errors.email.message}
              </p>
            )}
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="password">Password</Label>
              <Link
                to="/password-reset"
                className="text-sm text-primary hover:underline"
              >
                Forgot password?
              </Link>
            </div>
            <Input
              id="password"
              type="password"
              {...form.register('password')}
            />
          </div>

          <div className="flex items-center space-x-2">
            <Checkbox id="remember" {...form.register('remember')} />
            <Label htmlFor="remember" className="text-sm">
              Remember me for 30 days
            </Label>
          </div>

          <Button type="submit" className="w-full" disabled={isLoggingIn}>
            {isLoggingIn ? 'Signing in...' : 'Sign in'}
          </Button>
        </form>

        <div className="mt-6">
          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <span className="w-full border-t" />
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-background px-2 text-muted-foreground">
                Or continue with
              </span>
            </div>
          </div>

          <Link to="/magic-link">
            <Button variant="outline" className="w-full mt-4">
              Sign in with Magic Link
            </Button>
          </Link>
        </div>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          Don't have an account?{' '}
          <Link to="/register" className="text-primary hover:underline">
            Sign up
          </Link>
        </p>
      </CardContent>
    </Card>
  );
}
```

Add loading states and error handling.
```

---

## Prompt 7.2: Registration Page

```text
Create the registration page with password validation.

Create src/pages/auth/RegisterPage.tsx:
```typescript
import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { apiClient } from '@/api/client';
import { useAppToast } from '@/hooks/useToast';
import { registerSchema, RegisterInput } from '@/lib/validations/auth';
import { PasswordStrengthIndicator } from '@/components/PasswordStrengthIndicator';

export function RegisterPage() {
  const navigate = useNavigate();
  const { success, apiError } = useAppToast();

  const form = useForm<RegisterInput>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      email: '',
      password: '',
      confirmPassword: '',
    },
  });

  const password = form.watch('password');

  const registerMutation = useMutation({
    mutationFn: async (data: RegisterInput) => {
      const response = await apiClient.post('/auth/register', {
        email: data.email,
        password: data.password,
      });
      return response.data;
    },
    onSuccess: () => {
      success('Account created!', 'Please check your email to verify your account.');
      navigate('/login');
    },
    onError: (error) => {
      apiError(error);
    },
  });

  const onSubmit = (data: RegisterInput) => {
    registerMutation.mutate(data);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Create an account</CardTitle>
        <CardDescription>
          Get started with a8n.tools
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              placeholder="you@example.com"
              {...form.register('email')}
            />
            {form.formState.errors.email && (
              <p className="text-sm text-destructive">
                {form.formState.errors.email.message}
              </p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              type="password"
              {...form.register('password')}
            />
            <PasswordStrengthIndicator password={password} />
            {form.formState.errors.password && (
              <p className="text-sm text-destructive">
                {form.formState.errors.password.message}
              </p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="confirmPassword">Confirm Password</Label>
            <Input
              id="confirmPassword"
              type="password"
              {...form.register('confirmPassword')}
            />
            {form.formState.errors.confirmPassword && (
              <p className="text-sm text-destructive">
                {form.formState.errors.confirmPassword.message}
              </p>
            )}
          </div>

          <Button
            type="submit"
            className="w-full"
            disabled={registerMutation.isPending}
          >
            {registerMutation.isPending ? 'Creating account...' : 'Create account'}
          </Button>
        </form>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          Already have an account?{' '}
          <Link to="/login" className="text-primary hover:underline">
            Sign in
          </Link>
        </p>

        <p className="mt-4 text-center text-xs text-muted-foreground">
          By creating an account, you agree to our{' '}
          <Link to="/terms" className="underline">Terms of Service</Link>
          {' '}and{' '}
          <Link to="/privacy" className="underline">Privacy Policy</Link>.
        </p>
      </CardContent>
    </Card>
  );
}
```

Create src/components/PasswordStrengthIndicator.tsx:
- Show strength bar (weak/fair/good/strong)
- Show checkmarks for each requirement met
```

---

## Prompt 7.3: Magic Link Pages

```text
Create magic link request and verification pages.

Create src/pages/auth/MagicLinkPage.tsx:
```typescript
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { apiClient } from '@/api/client';
import { magicLinkSchema, MagicLinkInput } from '@/lib/validations/auth';
import { Mail } from 'lucide-react';

export function MagicLinkPage() {
  const [submitted, setSubmitted] = useState(false);
  const [email, setEmail] = useState('');

  const form = useForm<MagicLinkInput>({
    resolver: zodResolver(magicLinkSchema),
    defaultValues: { email: '' },
  });

  const requestMutation = useMutation({
    mutationFn: async (data: MagicLinkInput) => {
      await apiClient.post('/auth/magic-link', data);
    },
    onSuccess: () => {
      setEmail(form.getValues('email'));
      setSubmitted(true);
    },
  });

  if (submitted) {
    return (
      <Card>
        <CardHeader>
          <div className="mx-auto mb-4 p-3 bg-primary/10 rounded-full">
            <Mail className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-center">Check your email</CardTitle>
          <CardDescription className="text-center">
            We sent a magic link to <strong>{email}</strong>
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground text-center">
            Click the link in the email to sign in. The link expires in 15 minutes.
          </p>
          <Button
            variant="outline"
            className="w-full"
            onClick={() => {
              setSubmitted(false);
              form.reset();
            }}
          >
            Try a different email
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Sign in with Magic Link</CardTitle>
        <CardDescription>
          We'll send you a link to sign in without a password
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={form.handleSubmit((data) => requestMutation.mutate(data))} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              placeholder="you@example.com"
              {...form.register('email')}
            />
            {form.formState.errors.email && (
              <p className="text-sm text-destructive">
                {form.formState.errors.email.message}
              </p>
            )}
          </div>

          <Button
            type="submit"
            className="w-full"
            disabled={requestMutation.isPending}
          >
            {requestMutation.isPending ? 'Sending...' : 'Send Magic Link'}
          </Button>
        </form>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          <Link to="/login" className="text-primary hover:underline">
            Sign in with password instead
          </Link>
        </p>
      </CardContent>
    </Card>
  );
}
```

Create src/pages/auth/MagicLinkVerifyPage.tsx:
- Get token from URL query param
- Call verify endpoint
- Handle success/error
- Redirect to dashboard on success
```

---

## Prompt 7.4: Password Reset Pages

```text
Create password reset request and completion pages.

Create src/pages/auth/PasswordResetPage.tsx:
```typescript
import { useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { apiClient } from '@/api/client';
import { useAppToast } from '@/hooks/useToast';
import { PasswordStrengthIndicator } from '@/components/PasswordStrengthIndicator';
import { magicLinkSchema, passwordResetSchema } from '@/lib/validations/auth';
import { CheckCircle } from 'lucide-react';

export function PasswordResetPage() {
  const [searchParams] = useSearchParams();
  const token = searchParams.get('token');

  // If no token, show request form
  // If token present, show reset form
  return token ? <ResetForm token={token} /> : <RequestForm />;
}

function RequestForm() {
  const [submitted, setSubmitted] = useState(false);
  const form = useForm({
    resolver: zodResolver(magicLinkSchema),
    defaultValues: { email: '' },
  });

  const requestMutation = useMutation({
    mutationFn: async (data: { email: string }) => {
      await apiClient.post('/auth/password-reset', data);
    },
    onSuccess: () => setSubmitted(true),
  });

  if (submitted) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-center">Check your email</CardTitle>
          <CardDescription className="text-center">
            If an account exists, we sent a password reset link.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link to="/login">
            <Button variant="outline" className="w-full">
              Back to login
            </Button>
          </Link>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Reset your password</CardTitle>
        <CardDescription>
          Enter your email and we'll send you a reset link
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={form.handleSubmit((data) => requestMutation.mutate(data))} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              {...form.register('email')}
            />
          </div>
          <Button type="submit" className="w-full" disabled={requestMutation.isPending}>
            {requestMutation.isPending ? 'Sending...' : 'Send Reset Link'}
          </Button>
        </form>
        <p className="mt-4 text-center text-sm">
          <Link to="/login" className="text-primary hover:underline">
            Back to login
          </Link>
        </p>
      </CardContent>
    </Card>
  );
}

function ResetForm({ token }: { token: string }) {
  const [completed, setCompleted] = useState(false);
  const { success, apiError } = useAppToast();

  const form = useForm({
    resolver: zodResolver(passwordResetSchema),
    defaultValues: { password: '', confirmPassword: '' },
  });

  const resetMutation = useMutation({
    mutationFn: async (data: { password: string }) => {
      await apiClient.post('/auth/password-reset/confirm', {
        token,
        new_password: data.password,
      });
    },
    onSuccess: () => {
      success('Password reset!', 'You can now sign in with your new password.');
      setCompleted(true);
    },
    onError: apiError,
  });

  if (completed) {
    return (
      <Card>
        <CardHeader>
          <div className="mx-auto mb-4 p-3 bg-green-100 rounded-full">
            <CheckCircle className="h-6 w-6 text-green-600" />
          </div>
          <CardTitle className="text-center">Password Reset Complete</CardTitle>
        </CardHeader>
        <CardContent>
          <Link to="/login">
            <Button className="w-full">Sign in</Button>
          </Link>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Set new password</CardTitle>
        <CardDescription>
          Enter your new password below
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={form.handleSubmit((data) => resetMutation.mutate(data))} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="password">New Password</Label>
            <Input id="password" type="password" {...form.register('password')} />
            <PasswordStrengthIndicator password={form.watch('password')} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="confirmPassword">Confirm Password</Label>
            <Input id="confirmPassword" type="password" {...form.register('confirmPassword')} />
          </div>
          <Button type="submit" className="w-full" disabled={resetMutation.isPending}>
            {resetMutation.isPending ? 'Resetting...' : 'Reset Password'}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
```
```

---

## Prompt 7.5: Auth API Hooks

```text
Create reusable hooks for authentication API calls.

Create src/api/auth.ts:
```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import { useAuthStore } from '@/stores/authStore';
import { useNavigate } from 'react-router-dom';

// Types
export interface LoginRequest {
  email: string;
  password: string;
  remember?: boolean;
}

export interface RegisterRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  user: {
    id: string;
    email: string;
    role: string;
    subscription_status: string;
    price_locked: boolean;
  };
}

// Hooks
export function useLogin() {
  const queryClient = useQueryClient();
  const { setUser } = useAuthStore();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: async (data: LoginRequest): Promise<AuthResponse> => {
      const response = await apiClient.post('/auth/login', data);
      return response.data.data;
    },
    onSuccess: (data) => {
      setUser(data.user);
      queryClient.invalidateQueries({ queryKey: ['currentUser'] });
    },
  });
}

export function useRegister() {
  return useMutation({
    mutationFn: async (data: RegisterRequest) => {
      const response = await apiClient.post('/auth/register', data);
      return response.data.data;
    },
  });
}

export function useLogout() {
  const queryClient = useQueryClient();
  const { logout } = useAuthStore();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: async () => {
      await apiClient.post('/auth/logout');
    },
    onSuccess: () => {
      logout();
      queryClient.clear();
      navigate('/login');
    },
  });
}

export function useRequestMagicLink() {
  return useMutation({
    mutationFn: async (email: string) => {
      await apiClient.post('/auth/magic-link', { email });
    },
  });
}

export function useVerifyMagicLink() {
  const queryClient = useQueryClient();
  const { setUser } = useAuthStore();

  return useMutation({
    mutationFn: async (token: string): Promise<AuthResponse> => {
      const response = await apiClient.post('/auth/magic-link/verify', { token });
      return response.data.data;
    },
    onSuccess: (data) => {
      setUser(data.user);
      queryClient.invalidateQueries({ queryKey: ['currentUser'] });
    },
  });
}

export function useRequestPasswordReset() {
  return useMutation({
    mutationFn: async (email: string) => {
      await apiClient.post('/auth/password-reset', { email });
    },
  });
}

export function useConfirmPasswordReset() {
  return useMutation({
    mutationFn: async (data: { token: string; new_password: string }) => {
      await apiClient.post('/auth/password-reset/confirm', data);
    },
  });
}

export function useCurrentUser() {
  const { setUser, setLoading } = useAuthStore();

  return useQuery({
    queryKey: ['currentUser'],
    queryFn: async () => {
      const response = await apiClient.get('/users/me');
      return response.data.data;
    },
    retry: false,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}
```

Update components to use these hooks.
```

---

## Prompt 7.6: Protected Route Component

```text
Create a robust protected route component.

Create src/components/ProtectedRoute.tsx:
```typescript
import { Navigate, useLocation } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';
import { LoadingScreen } from './LoadingScreen';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requireSubscription?: boolean;
  requireAdmin?: boolean;
}

export function ProtectedRoute({
  children,
  requireSubscription = false,
  requireAdmin = false,
}: ProtectedRouteProps) {
  const { user, isAuthenticated, isLoading } = useAuthStore();
  const location = useLocation();

  if (isLoading) {
    return <LoadingScreen />;
  }

  if (!isAuthenticated) {
    // Redirect to login with return URL
    const returnTo = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?returnTo=${returnTo}`} replace />;
  }

  if (requireAdmin && user?.role !== 'admin') {
    return <Navigate to="/403" replace />;
  }

  if (requireSubscription) {
    const hasAccess = ['active', 'grace_period'].includes(user?.subscription_status || '');
    if (!hasAccess) {
      return <Navigate to="/subscription-required" replace />;
    }
  }

  return <>{children}</>;
}
```

Create src/pages/errors/SubscriptionRequiredPage.tsx:
```typescript
import { Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Lock } from 'lucide-react';

export function SubscriptionRequiredPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 p-4">
      <Card className="max-w-md w-full">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 p-3 bg-orange-100 rounded-full">
            <Lock className="h-6 w-6 text-orange-600" />
          </div>
          <CardTitle>Subscription Required</CardTitle>
          <CardDescription>
            You need an active subscription to access this feature.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="bg-slate-100 rounded-lg p-4">
            <p className="font-medium">a8n.tools Subscription</p>
            <p className="text-2xl font-bold text-primary">$3/month</p>
            <p className="text-sm text-muted-foreground">
              Access all tools. Price locked forever.
            </p>
          </div>
          <Link to="/pricing">
            <Button className="w-full">View Pricing</Button>
          </Link>
          <Link to="/dashboard">
            <Button variant="outline" className="w-full">
              Back to Dashboard
            </Button>
          </Link>
        </CardContent>
      </Card>
    </div>
  );
}
```

Update router to use ProtectedRoute wrapper.
```

---

## Prompt 7.7: Session Management UI

```text
Create UI for viewing and managing active sessions.

Create src/pages/dashboard/SessionsPage.tsx:
```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAppToast } from '@/hooks/useToast';
import { Monitor, Smartphone, Globe, Trash2 } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface Session {
  id: string;
  device_info: string;
  ip_address: string;
  created_at: string;
  last_used_at: string;
  is_current: boolean;
}

export function SessionsPage() {
  const queryClient = useQueryClient();
  const { success, apiError } = useAppToast();

  const { data: sessions, isLoading } = useQuery({
    queryKey: ['sessions'],
    queryFn: async () => {
      const response = await apiClient.get('/users/me/sessions');
      return response.data.data.sessions as Session[];
    },
  });

  const revokeMutation = useMutation({
    mutationFn: async (sessionId: string) => {
      await apiClient.delete(`/users/me/sessions/${sessionId}`);
    },
    onSuccess: () => {
      success('Session revoked');
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
    },
    onError: apiError,
  });

  const revokeAllMutation = useMutation({
    mutationFn: async () => {
      await apiClient.delete('/users/me/sessions');
    },
    onSuccess: () => {
      success('All other sessions revoked');
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
    },
    onError: apiError,
  });

  const getDeviceIcon = (deviceInfo: string) => {
    if (deviceInfo.toLowerCase().includes('mobile')) {
      return <Smartphone className="h-5 w-5" />;
    }
    return <Monitor className="h-5 w-5" />;
  };

  if (isLoading) {
    return <div>Loading sessions...</div>;
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex justify-between items-center">
          <div>
            <CardTitle>Active Sessions</CardTitle>
            <CardDescription>
              Manage your logged-in devices
            </CardDescription>
          </div>
          <Button
            variant="outline"
            onClick={() => revokeAllMutation.mutate()}
            disabled={revokeAllMutation.isPending}
          >
            Sign out all other sessions
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {sessions?.map((session) => (
            <div
              key={session.id}
              className="flex items-center justify-between p-4 border rounded-lg"
            >
              <div className="flex items-center gap-4">
                {getDeviceIcon(session.device_info)}
                <div>
                  <p className="font-medium">
                    {session.device_info}
                    {session.is_current && (
                      <span className="ml-2 text-xs bg-green-100 text-green-800 px-2 py-0.5 rounded">
                        Current
                      </span>
                    )}
                  </p>
                  <p className="text-sm text-muted-foreground flex items-center gap-2">
                    <Globe className="h-3 w-3" />
                    {session.ip_address}
                    <span>•</span>
                    Last active {formatDistanceToNow(new Date(session.last_used_at))} ago
                  </p>
                </div>
              </div>
              {!session.is_current && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => revokeMutation.mutate(session.id)}
                  disabled={revokeMutation.isPending}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              )}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
```
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Login form works with valid credentials
- [ ] Login shows error for invalid credentials
- [ ] Registration validates password requirements
- [ ] Password strength indicator updates live
- [ ] Magic link request shows confirmation
- [ ] Magic link verification logs user in
- [ ] Password reset email sent (check logs)
- [ ] Password reset with token works
- [ ] Protected routes redirect to login
- [ ] Admin routes block non-admins
- [ ] Session list shows active sessions
- [ ] Session revocation works
- [ ] Return URL preserved through login

---

## Next Steps

Proceed to **[08-frontend-dashboard.md](./08-frontend-dashboard.md)** to build the user dashboard.
