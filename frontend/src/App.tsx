import { useEffect } from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'
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

// Dashboard pages
import { DashboardPage } from '@/pages/dashboard/DashboardPage'
import { ApplicationsPage } from '@/pages/dashboard/ApplicationsPage'
import { MembershipPage } from '@/pages/dashboard/MembershipPage'
import { CheckoutSuccessPage } from '@/pages/dashboard/CheckoutSuccessPage'
import { SettingsPage } from '@/pages/dashboard/SettingsPage'

// Admin pages
import { AdminDashboardPage } from '@/pages/admin/AdminDashboardPage'
import { AdminUsersPage } from '@/pages/admin/AdminUsersPage'
import { AdminMembershipsPage } from '@/pages/admin/AdminMembershipsPage'
import { AdminApplicationsPage } from '@/pages/admin/AdminApplicationsPage'
import { AdminAuditLogsPage } from '@/pages/admin/AdminAuditLogsPage'

// Error pages
import { NotFoundPage } from '@/pages/errors/NotFoundPage'
import { MembershipRequiredPage } from '@/pages/errors/MembershipRequiredPage'

// Protected route wrapper
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuthStore()

  // Refresh user data once on mount to get latest info from backend
  useEffect(() => {
    if (isAuthenticated) {
      useAuthStore.getState().refreshUser()
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
    return <Navigate to="/login" replace />
  }

  return <>{children}</>
}

// Admin route wrapper
function AdminRoute({ children }: { children: React.ReactNode }) {
  const { user, isAuthenticated, isLoading } = useAuthStore()

  // Refresh user data once on mount to get latest role from backend
  useEffect(() => {
    if (isAuthenticated) {
      useAuthStore.getState().refreshUser()
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
    return <Navigate to="/login" replace />
  }

  if (user?.role !== 'admin') {
    return <Navigate to="/dashboard" replace />
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
        <Route path="/admin/audit-logs" element={<AdminAuditLogsPage />} />
      </Route>

      {/* 404 */}
      <Route path="*" element={<NotFoundPage />} />
    </Routes>
  )
}
