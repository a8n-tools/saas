import { http, HttpResponse } from 'msw'

// Mock user data
export const mockUser = {
  id: '123e4567-e89b-12d3-a456-426614174000',
  email: 'test@example.com',
  role: 'subscriber' as const,
  email_verified: true,
  two_factor_enabled: false,
  membership_status: 'active' as const,
  membership_tier: 'personal' as const,
  price_locked: false,
  locked_price_id: null,
  locked_price_amount: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
}

export const mockAdminUser = {
  ...mockUser,
  id: '123e4567-e89b-12d3-a456-426614174001',
  email: 'admin@example.com',
  role: 'admin' as const,
  last_login_at: '2024-01-01T00:00:00Z',
  grace_period_end: null,
}

export const mockApplication = {
  id: 'app-1',
  slug: 'rus',
  display_name: 'RUS',
  description: 'URL shortener with QR generation.',
  icon_url: null,
  version: '1.0.0',
  source_code_url: null,
  subdomain: 'rus',
  is_accessible: true,
  maintenance_mode: false,
  maintenance_message: null,
}

export const mockMembership = {
  status: 'active' as const,
  price_locked: true,
  locked_price_amount: 300,
  current_period_end: '2026-04-01T00:00:00Z',
  cancel_at_period_end: false,
  grace_period_end: null,
}

export const mockAdminStats = {
  total_users: 100,
  active_members: 75,
  past_due_members: 5,
  grace_period_members: 2,
  total_applications: 2,
  active_applications: 2,
}

export const mockAdminApplication = {
  id: 'app-admin-1',
  name: 'rus',
  slug: 'rus',
  display_name: 'RUS',
  description: 'URL shortener',
  icon_url: null,
  is_active: true,
  maintenance_mode: false,
  maintenance_message: null,
  subdomain: 'rus',
  container_name: 'rus',
  health_check_url: null,
  version: '1.0.0',
  source_code_url: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
}

export const mockAdminMembership = {
  id: 'mem-1',
  user_id: mockUser.id,
  user_email: mockUser.email,
  stripe_subscription_id: 'sub_123',
  status: 'active',
  tier: 'personal',
  amount_cents: 300,
  current_period_start: '2024-01-01T00:00:00Z',
  current_period_end: '2026-04-01T00:00:00Z',
  cancel_at_period_end: false,
  created_at: '2024-01-01T00:00:00Z',
}

export const mockAuditLog = {
  id: 'log-1',
  actor_id: mockUser.id,
  actor_email: mockUser.email,
  actor_role: 'subscriber',
  actor_ip_address: '127.0.0.1',
  action: 'user_login',
  resource_type: 'user',
  resource_id: mockUser.id,
  old_values: null,
  new_values: null,
  metadata: null,
  is_admin_action: false,
  severity: 'info',
  created_at: '2024-01-01T00:00:00Z',
}

export const mockFeedbackSummary = {
  id: 'fb-001',
  name: 'Alice',
  email_masked: 'al***@example.com',
  subject: 'Login issue',
  tags: ['Bug'],
  message_excerpt: 'I cannot log in with my credentials.',
  status: 'new' as const,
  created_at: '2026-01-01T00:00:00Z',
  responded_at: null,
}

export const mockFeedbackDetail = {
  id: 'fb-001',
  name: 'Alice',
  email: 'alice@example.com',
  email_masked: 'al***@example.com',
  subject: 'Login issue',
  tags: ['Bug'],
  message: 'I cannot log in with my credentials.',
  page_path: '/login',
  status: 'new' as const,
  admin_response: null,
  responded_by: null,
  responded_at: null,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  attachments: [],
}

// Must match the base URL used by apiClient (config.apiUrl + '/v1')
const API_BASE = `${import.meta.env.VITE_API_URL || ''}/v1`

export const handlers = [
  // Login
  http.post(`${API_BASE}/auth/login`, async ({ request }) => {
    const body = await request.json() as { email: string; password: string }

    if (body.email === 'test@example.com' && body.password === 'password123') {
      return HttpResponse.json({
        success: true,
        data: {
          user: mockUser,
          access_token: 'mock-access-token',
        },
      })
    }

    if (body.email === 'admin@example.com' && body.password === 'password123') {
      return HttpResponse.json({
        success: true,
        data: {
          user: mockAdminUser,
          access_token: 'mock-access-token',
        },
      })
    }

    return HttpResponse.json(
      {
        success: false,
        error: {
          code: 'INVALID_CREDENTIALS',
          message: 'Invalid email or password',
        },
      },
      { status: 401 }
    )
  }),

  // Register
  http.post(`${API_BASE}/auth/register`, async ({ request }) => {
    const body = await request.json() as { email: string; password: string }

    if (body.email === 'existing@example.com') {
      return HttpResponse.json(
        {
          success: false,
          error: {
            code: 'EMAIL_EXISTS',
            message: 'Email already registered',
          },
        },
        { status: 409 }
      )
    }

    const newUser = {
      ...mockUser,
      id: '123e4567-e89b-12d3-a456-426614174002',
      email: body.email,
      membership_status: 'none' as const,
      membership_tier: null,
    }

    return HttpResponse.json({
      success: true,
      data: {
        user: newUser,
        access_token: 'mock-access-token',
      },
    })
  }),

  // Logout
  http.post(`${API_BASE}/auth/logout`, () => {
    return HttpResponse.json({
      success: true,
      data: null,
    })
  }),

  // Refresh token
  http.post(`${API_BASE}/auth/refresh`, () => {
    return HttpResponse.json({
      success: true,
      data: {
        user: mockUser,
        access_token: 'mock-refreshed-token',
      },
    })
  }),

  // Get current user
  http.get(`${API_BASE}/users/me`, () => {
    return HttpResponse.json({
      success: true,
      data: mockUser,
    })
  }),

  // Request magic link
  http.post(`${API_BASE}/auth/magic-link`, async ({ request }) => {
    const body = await request.json() as { email: string }

    if (!body.email.includes('@')) {
      return HttpResponse.json(
        {
          success: false,
          error: {
            code: 'VALIDATION_ERROR',
            message: 'Invalid email format',
          },
        },
        { status: 400 }
      )
    }

    return HttpResponse.json({
      success: true,
      data: {
        message: 'Magic link sent to your email',
      },
    })
  }),

  // Verify magic link
  http.post(`${API_BASE}/auth/magic-link/verify`, async ({ request }) => {
    const body = await request.json() as { token: string }

    if (body.token === 'valid-token') {
      return HttpResponse.json({
        success: true,
        data: {
          user: mockUser,
          access_token: 'mock-access-token',
        },
      })
    }

    return HttpResponse.json(
      {
        success: false,
        error: {
          code: 'INVALID_TOKEN',
          message: 'Invalid or expired token',
        },
      },
      { status: 400 }
    )
  }),

  // Request password reset
  http.post(`${API_BASE}/auth/password-reset`, async () => {
    // Always return success to prevent email enumeration
    return HttpResponse.json({
      success: true,
      data: {
        message: 'If an account exists with this email, a reset link has been sent',
      },
    })
  }),

  // Request email verification
  http.post(`${API_BASE}/users/me/email/verify`, () => {
    return HttpResponse.json({
      success: true,
      data: {
        message: 'Verification email sent. Please check your inbox.',
      },
    })
  }),

  // Confirm email verification
  http.post(`${API_BASE}/users/me/email/verify/confirm`, async ({ request }) => {
    const body = await request.json() as { token: string }

    if (body.token === 'valid-verify-token') {
      return HttpResponse.json({
        success: true,
        data: {
          message: 'Email verified successfully.',
        },
      })
    }

    return HttpResponse.json(
      {
        success: false,
        error: {
          code: 'INVALID_TOKEN',
          message: 'Invalid or expired token',
        },
      },
      { status: 400 }
    )
  }),

  // Confirm password reset
  http.post(`${API_BASE}/auth/password-reset/confirm`, async ({ request }) => {
    const body = await request.json() as { token: string; password: string }

    if (body.token === 'valid-reset-token') {
      return HttpResponse.json({
        success: true,
        data: {
          message: 'Password has been reset successfully',
        },
      })
    }

    return HttpResponse.json(
      {
        success: false,
        error: {
          code: 'INVALID_TOKEN',
          message: 'Invalid or expired token',
        },
      },
      { status: 400 }
    )
  }),

  // Applications
  http.get(`${API_BASE}/applications`, () => {
    return HttpResponse.json({
      success: true,
      data: { applications: [mockApplication] },
    })
  }),

  http.get(`${API_BASE}/applications/:slug`, () => {
    return HttpResponse.json({
      success: true,
      data: mockApplication,
    })
  }),

  // Memberships
  http.get(`${API_BASE}/memberships/me`, () => {
    return HttpResponse.json({
      success: true,
      data: mockMembership,
    })
  }),

  http.get(`${API_BASE}/memberships/payments`, () => {
    return HttpResponse.json({
      success: true,
      data: { items: [], total: 0, page: 1, total_pages: 1 },
    })
  }),

  http.post(`${API_BASE}/memberships/checkout`, () => {
    return HttpResponse.json({
      success: true,
      data: { checkout_url: 'https://checkout.stripe.com/test', session_id: 'cs_test' },
    })
  }),

  http.post(`${API_BASE}/memberships/subscribe`, () => {
    return HttpResponse.json({
      success: true,
      data: { message: 'Subscribed', membership_status: 'active', membership_tier: 'personal' },
    })
  }),

  http.post(`${API_BASE}/memberships/cancel`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  http.post(`${API_BASE}/memberships/cancel-now`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  http.post(`${API_BASE}/memberships/reactivate`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  // 2FA endpoints
  http.get(`${API_BASE}/auth/2fa/status`, () => {
    return HttpResponse.json({
      success: true,
      data: { enabled: false, recovery_codes_remaining: 0 },
    })
  }),

  http.post(`${API_BASE}/auth/2fa/setup`, () => {
    return HttpResponse.json({
      success: true,
      data: { otpauth_uri: 'otpauth://totp/test?secret=TESTSECRET', secret: 'TESTSECRET' },
    })
  }),

  http.post(`${API_BASE}/auth/2fa/confirm`, () => {
    return HttpResponse.json({
      success: true,
      data: { codes: ['AAAA-BBBB', 'CCCC-DDDD', 'EEEE-FFFF', 'GGGG-HHHH', 'IIII-JJJJ', 'KKKK-LLLL', 'MMMM-NNNN', 'OOOO-PPPP'] },
    })
  }),

  http.post(`${API_BASE}/auth/2fa/verify`, () => {
    return HttpResponse.json({
      success: true,
      data: { user: mockUser, access_token: 'mock-access-token' },
    })
  }),

  http.post(`${API_BASE}/auth/2fa/disable`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  http.post(`${API_BASE}/auth/2fa/recovery-codes`, () => {
    return HttpResponse.json({
      success: true,
      data: { codes: ['CODE-1111', 'CODE-2222', 'CODE-3333', 'CODE-4444', 'CODE-5555', 'CODE-6666', 'CODE-7777', 'CODE-8888'] },
    })
  }),

  // User settings
  http.put(`${API_BASE}/users/me/password`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  http.post(`${API_BASE}/users/me/email`, () => {
    return HttpResponse.json({
      success: true,
      data: { message: 'Verification email sent to your new address.', requires_relogin: false },
    })
  }),

  http.post(`${API_BASE}/users/me/email/confirm`, () => {
    return HttpResponse.json({ success: true, data: { message: 'Email confirmed.' } })
  }),

  // Admin endpoints
  http.get(`${API_BASE}/admin/stats`, () => {
    return HttpResponse.json({ success: true, data: mockAdminStats })
  }),

  http.get(`${API_BASE}/admin/users`, () => {
    return HttpResponse.json({
      success: true,
      data: { items: [{ ...mockUser, last_login_at: null, grace_period_end: null }], total: 1, page: 1, total_pages: 1 },
    })
  }),

  http.get(`${API_BASE}/admin/applications`, () => {
    return HttpResponse.json({
      success: true,
      data: { applications: [mockAdminApplication] },
    })
  }),

  http.get(`${API_BASE}/admin/memberships`, () => {
    return HttpResponse.json({
      success: true,
      data: { items: [mockAdminMembership], total: 1, page: 1, total_pages: 1 },
    })
  }),

  http.get(`${API_BASE}/admin/audit-logs`, () => {
    return HttpResponse.json({
      success: true,
      data: { items: [mockAuditLog], total: 1, page: 1, total_pages: 1 },
    })
  }),

  http.get(`${API_BASE}/admin/health`, () => {
    return HttpResponse.json({
      success: true,
      data: { health: { status: 'healthy', database: { status: 'healthy' }, uptime_seconds: 3600 } },
    })
  }),

  http.put(`${API_BASE}/admin/users/:userId/status`, () => {
    return HttpResponse.json({ success: true, data: mockAdminUser })
  }),

  http.put(`${API_BASE}/admin/users/:userId/role`, () => {
    return HttpResponse.json({ success: true, data: mockAdminUser })
  }),

  http.delete(`${API_BASE}/admin/users/:userId`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  http.post(`${API_BASE}/admin/users/:userId/reset-password`, () => {
    return HttpResponse.json({
      success: true,
      data: { message: 'Password reset', temporary_password: 'TempPass123!' },
    })
  }),

  http.get(`${API_BASE}/admin/users/:userId`, () => {
    return HttpResponse.json({ success: true, data: { ...mockUser, last_login_at: null, grace_period_end: null } })
  }),

  http.post(`${API_BASE}/admin/users/:userId/impersonate`, () => {
    return HttpResponse.json({ success: true, data: { message: 'Impersonation started' } })
  }),

  http.put(`${API_BASE}/admin/applications/:appId`, () => {
    return HttpResponse.json({ success: true, data: mockAdminApplication })
  }),

  http.post(`${API_BASE}/admin/applications`, () => {
    return HttpResponse.json({ success: true, data: { ...mockAdminApplication, id: 'app-new' } })
  }),

  http.delete(`${API_BASE}/admin/applications/:appId`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  http.put(`${API_BASE}/admin/applications/:appId/swap-order`, () => {
    return HttpResponse.json({ success: true, data: { applications: [mockAdminApplication] } })
  }),

  http.post(`${API_BASE}/admin/memberships/grant`, () => {
    return HttpResponse.json({
      success: true,
      data: { status: 'active', price_locked: false, locked_price_amount: null, current_period_end: '2026-04-01T00:00:00Z', cancel_at_period_end: false, grace_period_end: null },
    })
  }),

  http.post(`${API_BASE}/admin/memberships/revoke`, () => {
    return HttpResponse.json({ success: true, data: { message: 'Revoked' } })
  }),

  http.get(`${API_BASE}/admin/notifications`, () => {
    return HttpResponse.json({ success: true, data: [] })
  }),

  http.post(`${API_BASE}/admin/notifications/:notificationId/read`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  http.post(`${API_BASE}/admin/notifications/read-all`, () => {
    return HttpResponse.json({ success: true, data: null })
  }),

  // Feedback (admin)
  http.get(`${API_BASE}/admin/feedback`, () => {
    return HttpResponse.json({
      success: true,
      data: {
        items: [mockFeedbackSummary],
        total: 1,
        page: 1,
        total_pages: 1,
      },
    })
  }),

  // Static feedback routes must come before :feedbackId to avoid path param matching
  http.get(`${API_BASE}/admin/feedback/export`, () => {
    return new HttpResponse('id,name,email\r\nfb-001,Alice,al***@example.com\r\n', {
      headers: { 'Content-Type': 'text/csv' },
    })
  }),

  http.get(`${API_BASE}/admin/feedback/archive`, () => {
    return HttpResponse.json({
      success: true,
      data: {
        items: [],
        total: 0,
        page: 1,
        per_page: 20,
        total_pages: 1,
      },
    })
  }),

  http.post(`${API_BASE}/admin/feedback/archive/:archiveId/restore`, () => {
    return HttpResponse.json({ success: true, data: { ...mockFeedbackDetail, status: 'reviewed' } })
  }),

  http.get(`${API_BASE}/admin/feedback/:feedbackId`, () => {
    return HttpResponse.json({ success: true, data: mockFeedbackDetail })
  }),

  http.post(`${API_BASE}/admin/feedback/:feedbackId/respond`, () => {
    return HttpResponse.json({
      success: true,
      data: { ...mockFeedbackDetail, status: 'responded', admin_response: 'Thank you for your feedback!' },
    })
  }),

  http.put(`${API_BASE}/admin/feedback/:feedbackId/status`, () => {
    return HttpResponse.json({
      success: true,
      data: { ...mockFeedbackDetail, status: 'closed' },
    })
  }),

  http.delete(`${API_BASE}/admin/feedback/:feedbackId`, () => {
    return HttpResponse.json({ success: true, data: {} })
  }),

  // Feedback (public)
  http.post(`${API_BASE}/feedback`, () => {
    return HttpResponse.json({
      success: true,
      data: { id: 'fb-001', message: 'Feedback submitted' },
    })
  }),
]
