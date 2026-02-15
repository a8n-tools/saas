// User types
export type UserRole = 'subscriber' | 'admin'

export type MembershipStatus =
  | 'none'
  | 'active'
  | 'past_due'
  | 'canceled'
  | 'incomplete'

export type MembershipTier = 'personal' | 'business'

export interface User {
  id: string
  email: string
  role: UserRole
  email_verified: boolean
  membership_status: MembershipStatus
  membership_tier: MembershipTier | null
  price_locked: boolean
  locked_price_id: string | null
  locked_price_amount: number | null
  created_at: string
  updated_at: string
}

// Auth types
export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
}

export interface MagicLinkRequest {
  email: string
}

export interface MagicLinkVerifyRequest {
  token: string
}

export interface PasswordResetRequest {
  email: string
}

export interface PasswordResetConfirmRequest {
  token: string
  password: string
}

export interface AuthResponse {
  user: User
  access_token: string
}

// Membership types - matches API's MembershipResponse
export interface Membership {
  status: MembershipStatus
  price_locked: boolean
  locked_price_amount: number | null
  current_period_end: string | null
  cancel_at_period_end: boolean
  grace_period_end: string | null
}

export interface PaymentHistory {
  id: string
  amount: number
  currency: string
  status: string
  created_at: string
}

export interface CheckoutRequest {
  tier: MembershipTier
}

export interface CheckoutSessionResponse {
  checkout_url: string
  session_id: string
}

// Application types
export interface Application {
  id: string
  slug: string
  display_name: string
  description: string | null
  icon_url: string | null
  version: string | null
  source_code_url: string | null
  is_accessible: boolean
  maintenance_mode: boolean
  maintenance_message: string | null
}

// API response types
export interface ApiResponse<T> {
  success: boolean
  data: T
}

export interface ApiError {
  success: false
  error: {
    code: string
    message: string
    details?: Record<string, string[]>
  }
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  page_size?: number
  per_page?: number
  total_pages: number
}

// Admin types
export interface AuditLog {
  id: string
  user_id: string | null
  action: string
  resource_type: string
  resource_id: string | null
  ip_address: string | null
  user_agent: string | null
  details: Record<string, unknown> | null
  is_admin_action: boolean
  created_at: string
}

export interface AdminNotification {
  id: string
  type: string
  title: string
  message: string
  is_read: boolean
  created_at: string
}

export interface AdminStats {
  total_users: number
  active_memberships: number
  monthly_revenue: number
  past_due_count: number
}
