# 06 - Frontend Foundation

## Overview

This document contains prompts for setting up the React frontend with Vite, TypeScript, Tailwind CSS, and shadcn/ui.

## Prerequisites
- Completed backend API (01-05)
- Node.js 20+ installed

---

## Prompt 6.1: Create React Project with Vite

```text
Create a new React project using Vite with TypeScript.

Run in the project root:
```bash
npm create vite@latest frontend -- --template react-ts
cd frontend
npm install
```

Configure TypeScript (tsconfig.json):
- Strict mode enabled
- Path aliases: "@/*" -> "src/*"
- Target: ES2022
- Module: ESNext

Configure Vite (vite.config.ts):
```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '/v1'),
      },
    },
  },
})
```

Create project structure:
```
frontend/
├── src/
│   ├── api/           # API client and hooks
│   ├── components/    # Reusable UI components
│   │   └── ui/        # shadcn/ui components
│   ├── pages/         # Page components
│   ├── hooks/         # Custom React hooks
│   ├── stores/        # Zustand stores
│   ├── lib/           # Utility functions
│   ├── types/         # TypeScript types
│   ├── styles/        # Global styles
│   ├── App.tsx
│   └── main.tsx
├── public/
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```

Create .env.example:
```
VITE_API_URL=http://localhost:8080
VITE_APP_URL=http://localhost:5173
```

Verify: `npm run dev` starts the development server.
```

---

## Prompt 6.2: Configure Tailwind CSS and shadcn/ui

```text
Set up Tailwind CSS and shadcn/ui component library.

Install Tailwind:
```bash
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

Configure tailwind.config.js with a8n.tools theme:
```javascript
/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "#f97316",  // Orange
          foreground: "#ffffff",
          50: "#fff7ed",
          100: "#ffedd5",
          200: "#fed7aa",
          300: "#fdba74",
          400: "#fb923c",
          500: "#f97316",
          600: "#ea580c",
          700: "#c2410c",
          800: "#9a3412",
          900: "#7c2d12",
        },
        rust: {
          DEFAULT: "#b7410e",
          light: "#d4652d",
          dark: "#8a3009",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
```

Initialize shadcn/ui:
```bash
npx shadcn@latest init
```

Select options:
- Style: Default
- Base color: Slate
- CSS variables: Yes

Install essential shadcn components:
```bash
npx shadcn@latest add button card input label form toast
npx shadcn@latest add dropdown-menu avatar badge separator
npx shadcn@latest add dialog alert-dialog sheet
npx shadcn@latest add table tabs skeleton
```

Create src/styles/globals.css with CSS variables for theme.

Update src/main.tsx to import globals.css.
```

---

## Prompt 6.3: Set Up React Router

```text
Configure React Router for the application.

Install:
```bash
npm install react-router-dom
```

Create src/router/index.tsx:
```typescript
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import { RootLayout } from '@/components/layouts/RootLayout';
import { AuthLayout } from '@/components/layouts/AuthLayout';
import { DashboardLayout } from '@/components/layouts/DashboardLayout';
import { AdminLayout } from '@/components/layouts/AdminLayout';

// Public pages
import { HomePage } from '@/pages/HomePage';
import { PricingPage } from '@/pages/PricingPage';
import { LoginPage } from '@/pages/auth/LoginPage';
import { RegisterPage } from '@/pages/auth/RegisterPage';
import { MagicLinkPage } from '@/pages/auth/MagicLinkPage';
import { PasswordResetPage } from '@/pages/auth/PasswordResetPage';
import { TermsPage } from '@/pages/legal/TermsPage';
import { PrivacyPage } from '@/pages/legal/PrivacyPage';

// Protected pages
import { DashboardPage } from '@/pages/dashboard/DashboardPage';
import { AppsPage } from '@/pages/dashboard/AppsPage';
import { AccountPage } from '@/pages/dashboard/AccountPage';
import { SubscriptionPage } from '@/pages/dashboard/SubscriptionPage';

// Admin pages
import { AdminDashboardPage } from '@/pages/admin/AdminDashboardPage';
import { AdminUsersPage } from '@/pages/admin/AdminUsersPage';

// Error pages
import { NotFoundPage } from '@/pages/errors/NotFoundPage';
import { ForbiddenPage } from '@/pages/errors/ForbiddenPage';

const router = createBrowserRouter([
  {
    element: <RootLayout />,
    children: [
      // Public routes
      { path: '/', element: <HomePage /> },
      { path: '/pricing', element: <PricingPage /> },
      { path: '/terms', element: <TermsPage /> },
      { path: '/privacy', element: <PrivacyPage /> },

      // Auth routes
      {
        element: <AuthLayout />,
        children: [
          { path: '/login', element: <LoginPage /> },
          { path: '/register', element: <RegisterPage /> },
          { path: '/magic-link', element: <MagicLinkPage /> },
          { path: '/password-reset', element: <PasswordResetPage /> },
        ],
      },

      // Dashboard routes (protected)
      {
        path: '/dashboard',
        element: <DashboardLayout />,
        children: [
          { index: true, element: <DashboardPage /> },
          { path: 'apps', element: <AppsPage /> },
          { path: 'account', element: <AccountPage /> },
          { path: 'subscription', element: <SubscriptionPage /> },
        ],
      },

      // Admin routes (protected, admin only)
      {
        path: '/admin',
        element: <AdminLayout />,
        children: [
          { index: true, element: <AdminDashboardPage /> },
          { path: 'users', element: <AdminUsersPage /> },
          // ... more admin routes
        ],
      },

      // Error routes
      { path: '/403', element: <ForbiddenPage /> },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);

export function AppRouter() {
  return <RouterProvider router={router} />;
}
```

Create placeholder components for each page.

Update App.tsx to use AppRouter.
```

---

## Prompt 6.4: Create API Client with TanStack Query

```text
Set up API client and data fetching with TanStack Query.

Install:
```bash
npm install @tanstack/react-query axios
```

Create src/api/client.ts:
```typescript
import axios, { AxiosError, AxiosInstance } from 'axios';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export const apiClient: AxiosInstance = axios.create({
  baseURL: `${API_URL}/v1`,
  withCredentials: true,  // Send cookies
  headers: {
    'Content-Type': 'application/json',
  },
});

// Response interceptor for error handling
apiClient.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config;

    // If 401 and not refresh endpoint, try refresh
    if (error.response?.status === 401 && !originalRequest?.url?.includes('refresh')) {
      try {
        await apiClient.post('/auth/refresh');
        // Retry original request
        return apiClient(originalRequest!);
      } catch (refreshError) {
        // Redirect to login
        window.location.href = '/login';
        return Promise.reject(refreshError);
      }
    }

    return Promise.reject(error);
  }
);
```

Create src/api/types.ts:
```typescript
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
  meta: {
    request_id: string;
    timestamp: string;
  };
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}
```

Create src/lib/query-client.ts:
```typescript
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60,  // 1 minute
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});
```

Update main.tsx to wrap with QueryClientProvider.

Create src/hooks/useApiError.ts for error handling.
```

---

## Prompt 6.5: Create Zustand Auth Store

```text
Set up Zustand for state management, starting with auth.

Install:
```bash
npm install zustand
```

Create src/stores/authStore.ts:
```typescript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface User {
  id: string;
  email: string;
  role: 'subscriber' | 'admin';
  subscription_status: 'none' | 'active' | 'past_due' | 'canceled' | 'grace_period';
  price_locked: boolean;
  email_verified: boolean;
}

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  setUser: (user: User | null) => void;
  setLoading: (loading: boolean) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isAuthenticated: false,
      isLoading: true,

      setUser: (user) => set({
        user,
        isAuthenticated: !!user,
        isLoading: false,
      }),

      setLoading: (isLoading) => set({ isLoading }),

      logout: () => set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
      }),
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({ user: state.user }),
    }
  )
);
```

Create src/hooks/useAuth.ts:
```typescript
import { useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAuthStore } from '@/stores/authStore';
import { apiClient } from '@/api/client';

export function useAuth() {
  const { user, isAuthenticated, isLoading, setUser, setLoading, logout } = useAuthStore();
  const queryClient = useQueryClient();

  // Fetch current user on mount
  const { refetch: refetchUser } = useQuery({
    queryKey: ['currentUser'],
    queryFn: async () => {
      const response = await apiClient.get('/users/me');
      return response.data.data;
    },
    enabled: false,  // Only fetch manually
    retry: false,
  });

  useEffect(() => {
    const checkAuth = async () => {
      try {
        const { data } = await refetchUser();
        setUser(data);
      } catch {
        setUser(null);
      }
    };

    checkAuth();
  }, []);

  const loginMutation = useMutation({
    mutationFn: async (data: { email: string; password: string }) => {
      const response = await apiClient.post('/auth/login', data);
      return response.data.data;
    },
    onSuccess: (data) => {
      setUser(data.user);
      queryClient.invalidateQueries({ queryKey: ['currentUser'] });
    },
  });

  const logoutMutation = useMutation({
    mutationFn: async () => {
      await apiClient.post('/auth/logout');
    },
    onSuccess: () => {
      logout();
      queryClient.clear();
    },
  });

  return {
    user,
    isAuthenticated,
    isLoading,
    login: loginMutation.mutate,
    logout: logoutMutation.mutate,
    isLoggingIn: loginMutation.isPending,
    isLoggingOut: logoutMutation.isPending,
    loginError: loginMutation.error,
  };
}
```

Create protected route wrapper component.
```

---

## Prompt 6.6: Create Layout Components

```text
Create the main layout components.

Create src/components/layouts/RootLayout.tsx:
```typescript
import { Outlet } from 'react-router-dom';
import { Toaster } from '@/components/ui/toaster';

export function RootLayout() {
  return (
    <div className="min-h-screen bg-background">
      <Outlet />
      <Toaster />
    </div>
  );
}
```

Create src/components/layouts/AuthLayout.tsx:
```typescript
import { Outlet, Navigate } from 'react-router-dom';
import { useAuth } from '@/hooks/useAuth';
import { Logo } from '@/components/Logo';

export function AuthLayout() {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return <LoadingScreen />;
  }

  if (isAuthenticated) {
    return <Navigate to="/dashboard" replace />;
  }

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-slate-50">
      <div className="mb-8">
        <Logo />
      </div>
      <div className="w-full max-w-md px-4">
        <Outlet />
      </div>
    </div>
  );
}
```

Create src/components/layouts/DashboardLayout.tsx:
```typescript
import { Outlet, Navigate } from 'react-router-dom';
import { useAuth } from '@/hooks/useAuth';
import { DashboardNav } from '@/components/DashboardNav';
import { DashboardHeader } from '@/components/DashboardHeader';

export function DashboardLayout() {
  const { isAuthenticated, isLoading, user } = useAuth();

  if (isLoading) {
    return <LoadingScreen />;
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return (
    <div className="min-h-screen bg-slate-50">
      <DashboardHeader user={user!} />
      <div className="flex">
        <DashboardNav />
        <main className="flex-1 p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
```

Create src/components/layouts/AdminLayout.tsx:
- Similar to DashboardLayout
- Check user.role === 'admin'
- Redirect to /403 if not admin
- Different navigation

Create supporting components:
- Logo.tsx
- DashboardNav.tsx
- DashboardHeader.tsx
- AdminNav.tsx
- LoadingScreen.tsx
```

---

## Prompt 6.7: Create Form Components with React Hook Form

```text
Set up form handling with React Hook Form and Zod.

Install:
```bash
npm install react-hook-form @hookform/resolvers zod
```

Create src/lib/validations/auth.ts:
```typescript
import { z } from 'zod';

export const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(1, 'Password is required'),
  remember: z.boolean().optional(),
});

export const registerSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z
    .string()
    .min(12, 'Password must be at least 12 characters')
    .regex(/[A-Z]/, 'Password must contain uppercase letter')
    .regex(/[a-z]/, 'Password must contain lowercase letter')
    .regex(/[0-9]/, 'Password must contain a number')
    .regex(/[^A-Za-z0-9]/, 'Password must contain a special character'),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});

export const magicLinkSchema = z.object({
  email: z.string().email('Invalid email address'),
});

export const passwordResetSchema = z.object({
  password: z
    .string()
    .min(12, 'Password must be at least 12 characters'),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});

export type LoginInput = z.infer<typeof loginSchema>;
export type RegisterInput = z.infer<typeof registerSchema>;
export type MagicLinkInput = z.infer<typeof magicLinkSchema>;
export type PasswordResetInput = z.infer<typeof passwordResetSchema>;
```

Create reusable form field component:
```typescript
// src/components/forms/FormField.tsx
import { useFormContext } from 'react-hook-form';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface FormFieldProps {
  name: string;
  label: string;
  type?: string;
  placeholder?: string;
}

export function FormField({ name, label, type = 'text', placeholder }: FormFieldProps) {
  const { register, formState: { errors } } = useFormContext();

  return (
    <div className="space-y-2">
      <Label htmlFor={name}>{label}</Label>
      <Input
        id={name}
        type={type}
        placeholder={placeholder}
        {...register(name)}
      />
      {errors[name] && (
        <p className="text-sm text-destructive">
          {errors[name]?.message as string}
        </p>
      )}
    </div>
  );
}
```

Create src/components/forms/PasswordInput.tsx with show/hide toggle.
```

---

## Prompt 6.8: Create Toast Notifications System

```text
Implement toast notifications for user feedback.

The shadcn toast is already installed. Create wrapper hooks.

Create src/hooks/useToast.ts (extend shadcn):
```typescript
import { useToast as useShadcnToast } from '@/components/ui/use-toast';

export function useAppToast() {
  const { toast } = useShadcnToast();

  return {
    success: (message: string, description?: string) => {
      toast({
        title: message,
        description,
        variant: 'default',
      });
    },

    error: (message: string, description?: string) => {
      toast({
        title: message,
        description,
        variant: 'destructive',
      });
    },

    loading: (message: string) => {
      return toast({
        title: message,
        description: 'Please wait...',
      });
    },

    apiError: (error: unknown) => {
      const message = extractErrorMessage(error);
      toast({
        title: 'Error',
        description: message,
        variant: 'destructive',
      });
    },
  };
}

function extractErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  // Handle axios error
  if (typeof error === 'object' && error !== null) {
    const axiosError = error as any;
    if (axiosError.response?.data?.error?.message) {
      return axiosError.response.data.error.message;
    }
  }
  return 'An unexpected error occurred';
}
```

Create error boundary component:
```typescript
// src/components/ErrorBoundary.tsx
import { Component, ErrorInfo, ReactNode } from 'react';
import { Button } from '@/components/ui/button';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
    // Send to error tracking service
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="flex flex-col items-center justify-center min-h-[400px]">
          <h2 className="text-xl font-semibold mb-4">Something went wrong</h2>
          <Button onClick={() => window.location.reload()}>
            Reload Page
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
```

Wrap App with ErrorBoundary.
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Vite dev server runs at localhost:5173
- [ ] TypeScript compiles without errors
- [ ] Tailwind styles apply correctly
- [ ] shadcn/ui components render
- [ ] Routes work for all pages
- [ ] API client can reach backend
- [ ] Auth store persists across refreshes
- [ ] Protected routes redirect correctly
- [ ] Toast notifications display
- [ ] Forms validate with Zod

---

## Next Steps

Proceed to **[07-frontend-auth.md](./07-frontend-auth.md)** to implement authentication UI.
