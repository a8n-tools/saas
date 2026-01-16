import { http, HttpResponse } from 'msw'

// Mock user data
export const mockUser = {
  id: '123e4567-e89b-12d3-a456-426614174000',
  email: 'test@example.com',
  role: 'subscriber' as const,
  email_verified: true,
  subscription_status: 'active' as const,
  subscription_tier: 'personal' as const,
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
}

// Base API URL used by the client
const API_BASE = '/api'

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
      subscription_status: 'none' as const,
      subscription_tier: null,
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
  http.post(`${API_BASE}/auth/password-reset`, async ({ request }) => {
    const body = await request.json() as { email: string }

    // Always return success to prevent email enumeration
    return HttpResponse.json({
      success: true,
      data: {
        message: 'If an account exists with this email, a reset link has been sent',
      },
    })
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
]
