import { useEffect, useState } from 'react'
import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { useEmailConfigStore } from '@/stores/emailConfigStore'
import { useStripeConfigStore } from '@/stores/stripeConfigStore'
import { authApi } from '@/api'
import { TooltipProvider } from '@/components/ui/tooltip'

// Layouts
import { PublicLayout } from '@/components/layout/PublicLayout'
import { DashboardLayout } from '@/components/layout/DashboardLayout'
import { AdminLayout } from '@/components/layout/AdminLayout'
import { FeedbackLauncher } from '@/components/layout/FeedbackLauncher'

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
import { OurStoryPage } from '@/pages/public/OurStoryPage'
import { AcceptInvitePage } from '@/pages/public/AcceptInvitePage'
import { SetupPage } from '@/pages/public/SetupPage'

// Dashboard pages
import { BillingPage } from '@/pages/dashboard/BillingPage'
import { DashboardPage } from '@/pages/dashboard/DashboardPage'
import { ApplicationsPage } from '@/pages/dashboard/ApplicationsPage'
import { MembershipPage } from '@/pages/dashboard/MembershipPage'
import { CheckoutSuccessPage } from '@/pages/dashboard/CheckoutSuccessPage'
import { SettingsPage } from '@/pages/dashboard/SettingsPage'
import { TwoFactorSetupPage } from '@/pages/dashboard/TwoFactorSetupPage'
import { DownloadsPage } from '@/pages/dashboard/DownloadsPage'

// Admin pages
import { AdminDashboardPage } from '@/pages/admin/AdminDashboardPage'
import { AdminUsersPage } from '@/pages/admin/AdminUsersPage'
import { AdminMembershipsPage } from '@/pages/admin/AdminMembershipsPage'
import { AdminApplicationsPage } from '@/pages/admin/AdminApplicationsPage'
import { AdminAuditLogsPage } from '@/pages/admin/AdminAuditLogsPage'
import { AdminFeedbackPage } from '@/pages/admin/AdminFeedbackPage'
import { AdminStripePage } from '@/pages/admin/AdminStripePage'
import AdminTierSettingsPage from '@/pages/admin/AdminTierSettingsPage'

// Settings pages (public, token-based)
import { ConfirmEmailPage } from '@/pages/settings/ConfirmEmailPage'
import { VerifyEmailPage } from '@/pages/settings/VerifyEmailPage'

// Error pages
import { NotFoundPage } from '@/pages/errors/NotFoundPage'
import { MembershipRequiredPage } from '@/pages/errors/MembershipRequiredPage'

// Redirect to /setup if no admin exists
function SetupGuard({ children }: { children: React.ReactNode }) {
  const location = useLocation()
  const [status, setStatus] = useState<'loading' | 'setup' | 'ready'>('loading')

  useEffect(() => {
    authApi.setupStatus()
      .then((res) => {
        useEmailConfigStore.getState().setEmailEnabled(res.email_enabled)
        useStripeConfigStore.getState().setStripeEnabled(res.stripe_enabled)
        setStatus(res.setup_required ? 'setup' : 'ready')
      })
      .catch(() => setStatus('ready'))
  }, [])

  if (status === 'loading') {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>
    )
  }

  if (status === 'setup' && location.pathname !== '/setup') {
    return <Navigate to="/setup" replace />
  }

  return <>{children}</>
}

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
    <TooltipProvider>
    <SetupGuard>
      <FeedbackLauncher />
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
          <Route path="/our-story" element={<OurStoryPage />} />
          <Route path="/feedback" element={<FeedbackPage />} />
          <Route path="/invite/accept" element={<AcceptInvitePage />} />
          <Route path="/setup" element={<SetupPage />} />
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
          <Route path="/billing" element={<BillingPage />} />
          <Route path="/membership" element={<MembershipPage />} />
          <Route path="/checkout/success" element={<CheckoutSuccessPage />} />
          <Route path="/downloads" element={<DownloadsPage />} />
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
          <Route path="/admin/stripe" element={<AdminStripePage />} />
          <Route path="/admin/tier-settings" element={<AdminTierSettingsPage />} />
          <Route path="/admin/feedback" element={<AdminFeedbackPage />} />
          <Route path="/admin/audit-logs" element={<AdminAuditLogsPage />} />
        </Route>

        {/* 404 */}
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </SetupGuard>
    </TooltipProvider>
  )
}
