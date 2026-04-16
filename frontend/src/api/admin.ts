import { apiClient } from './client'
import { config } from '@/config'
import type {
  User,
  Membership,
  AdminNotification,
  PaginatedResponse,
  FeedbackStatus,
  StripeProduct,
  StripePrice,
  StripeWebhookEndpoint,
} from '@/types'

// Actual stats response from API
export interface AdminStatsResponse {
  total_users: number
  active_members: number
  past_due_members: number
  grace_period_members: number
  total_applications: number
  active_applications: number
}

export interface AdminUser extends User {
  last_login_at: string | null
  grace_period_end: string | null
}

export interface AdminMembership {
  user_id: string
  user_email: string
  stripe_customer_id: string | null
  status: string
  subscription_tier: string
  subscription_override_by: string | null
  created_at: string
}

export interface AdminAuditLog {
  id: string
  actor_id: string | null
  actor_email: string | null
  actor_role: string | null
  actor_ip_address: string | null
  action: string
  resource_type: string | null
  resource_id: string | null
  old_values: Record<string, unknown> | null
  new_values: Record<string, unknown> | null
  metadata: Record<string, unknown> | null
  is_admin_action: boolean
  severity: string
  created_at: string
}

export interface UpdateUserStatusRequest {
  is_active: boolean
}

export interface UpdateUserRoleRequest {
  role: 'subscriber' | 'admin'
}

export interface AdminApplication {
  id: string
  name: string
  slug: string
  display_name: string
  description: string | null
  icon_url: string | null
  is_active: boolean
  maintenance_mode: boolean
  maintenance_message: string | null
  subdomain: string | null
  container_name: string
  health_check_url: string | null
  webhook_url: string | null
  version: string | null
  source_code_url: string | null
  forgejo_owner: string | null
  forgejo_repo: string | null
  pinned_release_tag: string | null
  oci_image_owner: string | null
  oci_image_name: string | null
  pinned_image_tag: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface UpdateApplicationRequest {
  display_name?: string
  description?: string
  icon_url?: string
  source_code_url?: string
  version?: string
  subdomain?: string
  container_name?: string
  health_check_url?: string
  webhook_url?: string
  is_active?: boolean
  maintenance_mode?: boolean
  maintenance_message?: string
  forgejo_owner?: string | null
  forgejo_repo?: string | null
  pinned_release_tag?: string | null
  oci_image_owner?: string | null
  oci_image_name?: string | null
  pinned_image_tag?: string | null
}

export interface CreateApplicationRequest {
  name: string
  slug: string
  display_name: string
  description?: string
  icon_url?: string
  container_name: string
  health_check_url?: string
  subdomain?: string
  webhook_url?: string
  version?: string
  source_code_url?: string
}

export interface DeleteApplicationRequest {
  password: string
  totp_code: string
}

export type AdminFeedbackStatus = FeedbackStatus

export interface AdminFeedbackSummary {
  id: string
  name: string | null
  email_masked: string | null
  subject: string | null
  tags: string[]
  message_excerpt: string
  status: AdminFeedbackStatus
  created_at: string
  responded_at: string | null
}

export interface AdminFeedbackDetail {
  id: string
  name: string | null
  email: string | null
  email_masked: string | null
  subject: string | null
  tags: string[]
  message: string
  page_path: string | null
  status: AdminFeedbackStatus
  admin_response: string | null
  responded_by: string | null
  responded_at: string | null
  created_at: string
  updated_at: string
  attachments: FeedbackAttachmentMeta[]
}

export interface FeedbackAttachmentMeta {
  id: string
  feedback_id: string
  filename: string
  mime_type: string
  size_bytes: number
  created_at: string
}

export interface AdminInvite {
  id: string
  email: string
  role: string
  invited_by: string
  expires_at: string
  accepted_at: string | null
  revoked_at: string | null
  created_at: string
}

export interface ArchivedFeedbackItem {
  id: string
  archived_at: string
  name: string | null
  email: string | null
  subject: string | null
  tags: string[]
  message_excerpt: string
  original_status: string | null
  created_at: string | null
}

export interface RespondToFeedbackRequest {
  response: string
  status?: AdminFeedbackStatus
}

export interface StripeConfigResponse {
  secret_key_masked: string | null
  webhook_secret_masked: string | null
  has_secret_key: boolean
  has_webhook_secret: boolean
  app_tag: string
  updated_at: string | null
  source: 'database' | 'environment'
}

export interface UpdateStripeConfigRequest {
  secret_key?: string
  webhook_secret?: string
  app_tag?: string
}

export interface TierConfigResponse {
  lifetime_slots: number
  early_adopter_slots: number
  early_adopter_trial_days: number
  standard_trial_days: number
  source: 'database' | 'environment'
  lifetime_slots_used: number
  early_adopter_slots_used: number
  updated_at: string
  updated_by: string | null
}

export interface UpdateTierConfigRequest {
  lifetime_slots?: number
  early_adopter_slots?: number
  early_adopter_trial_days?: number
  standard_trial_days?: number
}

export interface GrantMembershipRequest {
  user_id: string
  price_locked?: boolean
  locked_price_amount?: number
}

export interface RevokeMembershipRequest {
  user_id: string
}

export async function downloadFeedbackExport(): Promise<void> {
  const response = await fetch(`${config.apiUrl}/v1/admin/feedback/export`, { credentials: 'include' })
  if (!response.ok) throw new Error('Export failed')
  const blob = await response.blob()
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'feedback.csv'
  a.click()
  URL.revokeObjectURL(url)
}

export function getFeedbackAttachmentUrl(feedbackId: string, attachmentId: string): string {
  return `${config.apiUrl}/v1/admin/feedback/${feedbackId}/attachments/${attachmentId}`
}

export const adminApi = {
  // Stats
  getStats: (): Promise<AdminStatsResponse> =>
    apiClient.get('/admin/stats'),

  // Users
  getUsers: (page = 1, pageSize = 20, search?: string): Promise<PaginatedResponse<AdminUser>> => {
    const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) })
    if (search) params.append('search', search)
    return apiClient.get(`/admin/users?${params}`)
  },

  getUser: (userId: string): Promise<AdminUser> =>
    apiClient.get(`/admin/users/${userId}`),

  updateUserStatus: (userId: string, data: UpdateUserStatusRequest): Promise<AdminUser> =>
    apiClient.put(`/admin/users/${userId}/status`, data),

  deleteUser: (userId: string): Promise<void> =>
    apiClient.delete(`/admin/users/${userId}`),

  updateUserRole: (userId: string, data: UpdateUserRoleRequest): Promise<AdminUser> =>
    apiClient.put(`/admin/users/${userId}/role`, data),

  resetUserPassword: (userId: string): Promise<{ message: string; temporary_password: string }> =>
    apiClient.post(`/admin/users/${userId}/reset-password`),

  impersonateUser: (userId: string): Promise<{ message: string }> =>
    apiClient.post(`/admin/users/${userId}/impersonate`),

  // Memberships
  getMemberships: (page = 1, pageSize = 20, status?: string): Promise<PaginatedResponse<AdminMembership>> => {
    const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) })
    if (status) params.append('status', status)
    return apiClient.get(`/admin/memberships?${params}`)
  },

  grantMembership: (data: GrantMembershipRequest): Promise<Membership> =>
    apiClient.post('/admin/memberships/grant', data),

  revokeMembership: (data: RevokeMembershipRequest): Promise<{ message: string }> =>
    apiClient.post('/admin/memberships/revoke', data),

  // Applications
  getApplications: async (): Promise<AdminApplication[]> => {
    const response = await apiClient.get<{ applications: AdminApplication[] }>('/admin/applications')
    return response.applications
  },

  updateApplication: (appId: string, data: UpdateApplicationRequest): Promise<AdminApplication> =>
    apiClient.put(`/admin/applications/${appId}`, data),

  createApplication: (data: CreateApplicationRequest): Promise<AdminApplication> =>
    apiClient.post('/admin/applications', data),

  deleteApplication: (appId: string, data: DeleteApplicationRequest): Promise<void> =>
    apiClient.delete(`/admin/applications/${appId}`, data),

  swapApplicationOrder: async (appId: string, targetAppId: string): Promise<AdminApplication[]> => {
    const response = await apiClient.put<{ applications: AdminApplication[] }>(
      `/admin/applications/${appId}/swap-order`,
      { target_app_id: targetAppId }
    )
    return response.applications
  },

  // Audit Logs
  getAuditLogs: (
    page = 1,
    pageSize = 50,
    filters?: { action?: string; actor_id?: string; admin_only?: boolean }
  ): Promise<PaginatedResponse<AdminAuditLog>> => {
    const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) })
    if (filters?.action) params.append('action', filters.action)
    if (filters?.actor_id) params.append('actor_id', filters.actor_id)
    if (filters?.admin_only) params.append('admin_only', 'true')
    return apiClient.get(`/admin/audit-logs?${params}`)
  },

  // Feedback
  getFeedback: (
    page = 1,
    pageSize = 20,
    status?: AdminFeedbackStatus,
  ): Promise<PaginatedResponse<AdminFeedbackSummary>> => {
    const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) })
    if (status) params.append('status', status)
    return apiClient.get(`/admin/feedback?${params}`)
  },

  getFeedbackDetail: (feedbackId: string): Promise<AdminFeedbackDetail> =>
    apiClient.get(`/admin/feedback/${feedbackId}`),

  respondToFeedback: (feedbackId: string, data: RespondToFeedbackRequest): Promise<AdminFeedbackDetail> =>
    apiClient.post(`/admin/feedback/${feedbackId}/respond`, data),

  updateFeedbackStatus: (feedbackId: string, status: AdminFeedbackStatus): Promise<AdminFeedbackDetail> =>
    apiClient.put(`/admin/feedback/${feedbackId}/status`, { status }),

  deleteFeedback: (feedbackId: string): Promise<void> =>
    apiClient.delete(`/admin/feedback/${feedbackId}`),

  getArchivedFeedback: (page = 1, pageSize = 20): Promise<PaginatedResponse<ArchivedFeedbackItem>> => {
    const params = new URLSearchParams({ page: String(page), per_page: String(pageSize) })
    return apiClient.get(`/admin/feedback/archive?${params}`)
  },

  restoreFeedback: (archiveId: string): Promise<AdminFeedbackDetail> =>
    apiClient.post(`/admin/feedback/archive/${archiveId}/restore`),

  // Notifications
  getNotifications: (): Promise<AdminNotification[]> =>
    apiClient.get('/admin/notifications'),

  markNotificationRead: (notificationId: string): Promise<void> =>
    apiClient.post(`/admin/notifications/${notificationId}/read`),

  markAllNotificationsRead: (): Promise<void> =>
    apiClient.post('/admin/notifications/read-all'),

  // Admin Invites
  createInvite: (data: { email: string }): Promise<{ email: string }> =>
    apiClient.post('/admin/invites', data),

  getInvites: (page = 1, pageSize = 20): Promise<PaginatedResponse<AdminInvite>> => {
    const params = new URLSearchParams({ page: String(page), per_page: String(pageSize) })
    return apiClient.get(`/admin/invites?${params}`)
  },

  revokeInvite: (inviteId: string): Promise<void> =>
    apiClient.delete(`/admin/invites/${inviteId}`),

  // Tier config
  getTierConfig: (): Promise<TierConfigResponse> =>
    apiClient.get('/admin/tier-config'),

  updateTierConfig: (data: UpdateTierConfigRequest): Promise<TierConfigResponse> =>
    apiClient.put('/admin/tier-config', data),

  // Stripe config
  getStripeConfig: (): Promise<StripeConfigResponse> =>
    apiClient.get('/admin/stripe'),

  updateStripeConfig: (data: UpdateStripeConfigRequest): Promise<StripeConfigResponse> =>
    apiClient.put('/admin/stripe', data),

  // Stripe Products
  listStripeProducts: (): Promise<StripeProduct[]> =>
    apiClient.get('/admin/stripe/products'),

  createStripeProduct: (data: { name: string; description?: string; metadata?: Record<string, string> }): Promise<StripeProduct> =>
    apiClient.post('/admin/stripe/products', data),

  updateStripeProduct: (id: string, data: { name?: string; description?: string; metadata?: Record<string, string>; active?: boolean }): Promise<StripeProduct> =>
    apiClient.put(`/admin/stripe/products/${id}`, data),

  archiveStripeProduct: (id: string): Promise<void> =>
    apiClient.delete(`/admin/stripe/products/${id}`),

  // Stripe Prices
  listStripePrices: (productId?: string): Promise<StripePrice[]> =>
    apiClient.get(`/admin/stripe/prices${productId ? `?product_id=${productId}` : ''}`),

  createStripePrice: (data: { product_id: string; unit_amount: number; currency: string; interval: string }): Promise<StripePrice> =>
    apiClient.post('/admin/stripe/prices', data),

  archiveStripePrice: (id: string): Promise<void> =>
    apiClient.delete(`/admin/stripe/prices/${id}`),

  // Stripe Webhooks
  listStripeWebhooks: (): Promise<StripeWebhookEndpoint[]> =>
    apiClient.get('/admin/stripe/webhooks'),

  createStripeWebhook: (data: { url: string; enabled_events: string[] }): Promise<StripeWebhookEndpoint> =>
    apiClient.post('/admin/stripe/webhooks', data),

  deleteStripeWebhook: (id: string): Promise<void> =>
    apiClient.delete(`/admin/stripe/webhooks/${id}`),

  // Health
  getHealth: async (): Promise<{ status: string; database: string; uptime_seconds: number }> => {
    const response = await apiClient.get<{
      health: { status: string; database: { status: string }; uptime_seconds: number }
    }>('/admin/health')
    return {
      status: response.health.status,
      database: response.health.database.status,
      uptime_seconds: response.health.uptime_seconds,
    }
  },
}
