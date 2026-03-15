import { apiClient } from './client'
import type {
  Membership,
  PaymentHistory,
  CheckoutSessionResponse,
  CheckoutRequest,
  MembershipTier,
  PaginatedResponse,
} from '@/types'

export interface SubscribeResponse {
  message: string
  membership_status: string
  membership_tier: string
}

export const membershipApi = {
  getCurrent: (): Promise<Membership | null> =>
    apiClient.get('/memberships/me'),

  createCheckout: (tier: MembershipTier = 'personal'): Promise<CheckoutSessionResponse> =>
    apiClient.post('/memberships/checkout', { tier } as CheckoutRequest),

  subscribe: (tier: MembershipTier = 'personal'): Promise<SubscribeResponse> =>
    apiClient.post('/memberships/subscribe', { tier }),

  cancel: (): Promise<void> =>
    apiClient.post('/memberships/cancel'),

  cancelNow: (): Promise<void> =>
    apiClient.post('/memberships/cancel-now'),

  reactivate: (): Promise<void> =>
    apiClient.post('/memberships/reactivate'),

  getPaymentHistory: (
    page = 1,
    pageSize = 10
  ): Promise<PaginatedResponse<PaymentHistory>> =>
    apiClient.get(`/memberships/payments?page=${page}&page_size=${pageSize}`),
}
