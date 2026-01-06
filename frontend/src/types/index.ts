// User types
export type UserRole = 'subscriber' | 'admin'

export type SubscriptionStatus =
  | 'none'
  | 'active'
  | 'past_due'
  | 'canceled'
  | 'incomplete'

export type SubscriptionTier = 'personal' | 'business'

export interface User {
  id: string
  email: string
  role: UserRole
  email_verified: boolean
  subscription_status: SubscriptionStatus
  subscription_tier: SubscriptionTier | null
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

// Subscription types
export interface Subscription {
  id: string
  user_id: string
  stripe_subscription_id: string
  stripe_customer_id: string
  status: SubscriptionStatus
  tier: SubscriptionTier
  amount_cents: number
  current_period_start: string
  current_period_end: string
  cancel_at_period_end: boolean
  canceled_at: string | null
  created_at: string
  updated_at: string
}

export interface PaymentHistory {
  id: string
  user_id: string
  stripe_invoice_id: string
  stripe_payment_intent_id: string | null
  amount_cents: number
  currency: string
  status: string
  invoice_pdf_url: string | null
  created_at: string
}

export interface CheckoutRequest {
  tier: SubscriptionTier
}

export interface CheckoutSessionResponse {
  checkout_url: string
  session_id: string
}

// Application types
export interface Application {
  id: string
  name: string
  slug: string
  description: string
  icon_url: string | null
  subdomain: string
  is_active: boolean
  is_maintenance: boolean
  created_at: string
  updated_at: string
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
  page_size: number
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
  active_subscriptions: number
  monthly_revenue: number
  past_due_count: number
}
