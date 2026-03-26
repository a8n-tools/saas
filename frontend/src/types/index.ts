// User types
export type UserRole = 'subscriber' | 'admin'

export type MembershipStatus =
  | 'none'
  | 'active'
  | 'past_due'
  | 'canceled'
  | 'incomplete'
  | 'grace_period'

export type MembershipTier = 'personal' | 'business'

export type SubscriptionTier = 'lifetime' | 'trial_3m' | 'trial_1m'

export interface User {
  id: string
  email: string
  role: UserRole
  email_verified: boolean
  two_factor_enabled: boolean
  membership_status: MembershipStatus
  membership_tier: MembershipTier | null
  price_locked: boolean
  locked_price_id: string | null
  locked_price_amount: number | null
  created_at: string
  updated_at: string
  subscription_tier: SubscriptionTier
  trial_ends_at: string | null
  lifetime_member: boolean
}

// Auth types
export interface LoginRequest {
  email: string
  password: string
  remember?: boolean
}

export interface RegisterRequest {
  email: string
  password: string
  stripe_customer_id?: string
  payment_method_id?: string
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
  new_password: string
}

export interface AuthResponse {
  user: User
  access_token: string
}

export interface TwoFactorChallengeResponse {
  requires_2fa: true
  challenge_token: string
}

export interface TwoFactorSetupResponse {
  otpauth_uri: string
  secret: string
}

export interface RecoveryCodesResponse {
  codes: string[]
}

export interface TwoFactorStatusResponse {
  enabled: boolean
  recovery_codes_remaining: number
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

export interface CreateFeedbackRequest {
  name?: string
  email?: string
  subject?: string
  tags?: string[]
  message: string
  page_path?: string
  website?: string
}

export interface FeedbackSubmissionResponse {
  id: string
  message: string
}

export type FeedbackStatus = 'new' | 'reviewed' | 'responded' | 'closed'

// Application types
export interface Application {
  id: string
  slug: string
  display_name: string
  description: string | null
  icon_url: string | null
  version: string | null
  source_code_url: string | null
  subdomain: string | null
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

// Invoice types
export interface Invoice {
  id: string
  invoice_number: string
  amount_cents: number
  currency: string
  description: string
  created_at: string
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

// Stripe types
export interface StripeProduct {
  id: string
  name: string
  description: string | null
  active: boolean
  metadata: Record<string, string>
  created: number
}

export interface StripePrice {
  id: string
  product_id: string
  unit_amount: number | null
  currency: string
  recurring_interval: string | null
  active: boolean
}

export interface StripeWebhookEndpoint {
  id: string
  url: string
  enabled_events: string[]
  status: string
  secret?: string
}

export interface StripeInvoice {
  id: string
  amount_paid: number
  currency: string
  status: string | null
  invoice_pdf: string | null
  hosted_invoice_url: string | null
  created: number
  description: string | null
  number: string | null
}

export interface StripePaymentResponse {
  id: string
  amount: number
  currency: string
  status: string | null
  created: number
  invoice_pdf: string | null
}
