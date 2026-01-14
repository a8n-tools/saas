import { apiClient } from './client'
import type {
  User,
  Subscription,
  Application,
  AuditLog,
  AdminNotification,
  PaginatedResponse,
} from '@/types'

// Actual stats response from API
export interface AdminStatsResponse {
  total_users: number
  active_subscribers: number
  past_due_subscribers: number
  grace_period_subscribers: number
  total_applications: number
  active_applications: number
}

export interface AdminUser extends User {
  last_login_at: string | null
  grace_period_end: string | null
}

export interface AdminSubscription {
  id: string
  user_id: string
  user_email: string
  stripe_subscription_id: string | null
  status: string
  tier: string
  amount_cents: number
  current_period_start: string | null
  current_period_end: string | null
  cancel_at_period_end: boolean
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

export interface UpdateApplicationRequest {
  is_active?: boolean
  is_maintenance?: boolean
}

export interface GrantSubscriptionRequest {
  user_id: string
  tier: 'personal' | 'business'
  months?: number
}

export interface RevokeSubscriptionRequest {
  user_id: string
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

  // Subscriptions
  getSubscriptions: (page = 1, pageSize = 20, status?: string): Promise<PaginatedResponse<AdminSubscription>> => {
    const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) })
    if (status) params.append('status', status)
    return apiClient.get(`/admin/subscriptions?${params}`)
  },

  grantSubscription: (data: GrantSubscriptionRequest): Promise<Subscription> =>
    apiClient.post('/admin/subscriptions/grant', data),

  revokeSubscription: (data: RevokeSubscriptionRequest): Promise<{ message: string }> =>
    apiClient.post('/admin/subscriptions/revoke', data),

  // Applications
  getApplications: async (): Promise<Application[]> => {
    const response = await apiClient.get<{ applications: Application[] }>('/admin/applications')
    return response.applications
  },

  updateApplication: (appId: string, data: UpdateApplicationRequest): Promise<Application> =>
    apiClient.put(`/admin/applications/${appId}`, data),

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

  // Notifications
  getNotifications: (): Promise<AdminNotification[]> =>
    apiClient.get('/admin/notifications'),

  markNotificationRead: (notificationId: string): Promise<void> =>
    apiClient.post(`/admin/notifications/${notificationId}/read`),

  markAllNotificationsRead: (): Promise<void> =>
    apiClient.post('/admin/notifications/read-all'),

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
