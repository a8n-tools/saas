import { useEffect } from 'react'
import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'

// Layouts
import { PublicLayout } from '@/components/layout/PublicLayout'
import { DashboardLayout } from '@/components/layout/DashboardLayout'
import { AdminLayout } from '@/components/layout/AdminLayout'

// Public pages
import { LandingPage } from '@/pages/public/LandingPage'
import { PricingPage } from '@/pages/public/PricingPage'
import { LoginPage } from '@/pages/public/LoginPage'
import { RegisterPage } from '@/pages/public/RegisterPage'
import { MagicLinkPage } from '@/pages/public/MagicLinkPage'
import { PasswordResetPage } from '@/pages/public/PasswordResetPage'
import { PasswordResetConfirmPage } from '@/pages/public/PasswordResetConfirmPage'
import { TermsOfServicePage } from '@/pages/public/TermsOfServicePage'
import { PrivacyPolicyPage } from '@/pages/public/PrivacyPolicyPage'
import { TwoFactorVerifyPage } from '@/pages/public/TwoFactorVerifyPage'
import { LogoutPage } from '@/pages/public/LogoutPage'
import { FeedbackPage } from '@/pages/public/FeedbackPage'

// Dashboard pages
import { DashboardPage } from '@/pages/dashboard/DashboardPage'
import { ApplicationsPage } from '@/pages/dashboard/ApplicationsPage'
import { MembershipPage } from '@/pages/dashboard/MembershipPage'
import { CheckoutSuccessPage } from '@/pages/dashboard/CheckoutSuccessPage'
import { SettingsPage } from '@/pages/dashboard/SettingsPage'
import { TwoFactorSetupPage } from '@/pages/dashboard/TwoFactorSetupPage'

// Admin pages
import { AdminDashboardPage } from '@/pages/admin/AdminDashboardPage'
import { AdminUsersPage } from '@/pages/admin/AdminUsersPage'
import { AdminMembershipsPage } from '@/pages/admin/AdminMembershipsPage'
import { AdminApplicationsPage } from '@/pages/admin/AdminApplicationsPage'
import { AdminAuditLogsPage } from '@/pages/admin/AdminAuditLogsPage'
import { AdminFeedbackPage } from '@/pages/admin/AdminFeedbackPage'

// Settings pages (public, token-based)
import { ConfirmEmailPage } from '@/pages/settings/ConfirmEmailPage'
import { VerifyEmailPage } from '@/pages/settings/VerifyEmailPage'

// Error pages
import { NotFoundPage } from '@/pages/errors/NotFoundPage'
import { MembershipRequiredPage } from '@/pages/errors/MembershipRequiredPage'

// Protected route wrapper
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, isAuthenticated, isLoading } = useAuthStore()
  const location = useLocation()

  // Refresh user data on mount, or clear loading state if not authenticated
  useEffect(() => {
    if (isAuthenticated) {
      useAuthStore.getState().refreshUser()
    } else {
      useAuthStore.getState().setLoading(false)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>
    )
  }

  if (!isAuthenticated) {
    const redirect = location.pathname !== '/dashboard' ? `?redirect=${encodeURIComponent(location.pathname + location.search)}` : ''
    return <Navigate to={`/login${redirect}`} replace />
  }

  // Enforce 2FA for admin users (skip if already on the setup page)
  if (user?.role === 'admin' && !user.two_factor_enabled && location.pathname !== '/settings/2fa/setup') {
    return <Navigate to="/settings/2fa/setup" replace />
  }

  return <>{children}</>
}

// Admin route wrapper
function AdminRoute({ children }: { children: React.ReactNode }) {
  const { user, isAuthenticated, isLoading } = useAuthStore()
  const location = useLocation()

  // Refresh user data on mount, or clear loading state if not authenticated
  useEffect(() => {
    if (isAuthenticated) {
      useAuthStore.getState().refreshUser()
    } else {
      useAuthStore.getState().setLoading(false)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>
    )
  }

  if (!isAuthenticated) {
    const redirect = `?redirect=${encodeURIComponent(location.pathname + location.search)}`
    return <Navigate to={`/login${redirect}`} replace />
  }

  if (user?.role !== 'admin') {
    return <Navigate to="/dashboard" replace />
  }

  if (!user.two_factor_enabled) {
    return <Navigate to="/settings/2fa/setup" replace />
  }

  return <>{children}</>
}

export default function App() {
  return (
    <Routes>
      {/* Public routes */}
      <Route element={<PublicLayout />}>
        <Route path="/" element={<LandingPage />} />
        <Route path="/pricing" element={<PricingPage />} />
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/magic-link" element={<MagicLinkPage />} />
        <Route path="/password-reset" element={<PasswordResetPage />} />
        <Route path="/password-reset/confirm" element={<PasswordResetConfirmPage />} />
        <Route path="/terms" element={<TermsOfServicePage />} />
        <Route path="/privacy" element={<PrivacyPolicyPage />} />
        <Route path="/settings/confirm-email" element={<ConfirmEmailPage />} />
        <Route path="/settings/verify-email" element={<VerifyEmailPage />} />
        <Route path="/login/2fa" element={<TwoFactorVerifyPage />} />
        <Route path="/logout" element={<LogoutPage />} />
        <Route path="/feedback" element={<FeedbackPage />} />
      </Route>

      {/* Protected dashboard routes */}
      <Route
        element={
          <ProtectedRoute>
            <DashboardLayout />
          </ProtectedRoute>
        }
      >
        <Route path="/dashboard" element={<DashboardPage />} />
        <Route path="/applications" element={<ApplicationsPage />} />
        <Route path="/membership" element={<MembershipPage />} />
        <Route path="/checkout/success" element={<CheckoutSuccessPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/settings/2fa/setup" element={<TwoFactorSetupPage />} />
        <Route path="/membership-required" element={<MembershipRequiredPage />} />
      </Route>

      {/* Admin routes */}
      <Route
        element={
          <AdminRoute>
            <AdminLayout />
          </AdminRoute>
        }
      >
        <Route path="/admin" element={<AdminDashboardPage />} />
        <Route path="/admin/users" element={<AdminUsersPage />} />
        <Route path="/admin/memberships" element={<AdminMembershipsPage />} />
        <Route path="/admin/applications" element={<AdminApplicationsPage />} />
        <Route path="/admin/feedback" element={<AdminFeedbackPage />} />
        <Route path="/admin/audit-logs" element={<AdminAuditLogsPage />} />
      </Route>

      {/* 404 */}
      <Route path="*" element={<NotFoundPage />} />
    </Routes>
  )
}
