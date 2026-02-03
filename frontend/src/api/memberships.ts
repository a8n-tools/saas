import { apiClient } from './client'
import type {
  Membership,
  PaymentHistory,
  CheckoutSessionResponse,
  CheckoutRequest,
  MembershipTier,
  PaginatedResponse,
} from '@/types'

export const membershipApi = {
  getCurrent: (): Promise<Membership | null> =>
    apiClient.get('/memberships/me'),

  createCheckout: (tier: MembershipTier = 'personal'): Promise<CheckoutSessionResponse> =>
    apiClient.post('/memberships/checkout', { tier } as CheckoutRequest),

  cancel: (): Promise<void> =>
    apiClient.post('/memberships/cancel'),

  reactivate: (): Promise<void> =>
    apiClient.post('/memberships/reactivate'),

  getPaymentHistory: (
    page = 1,
    pageSize = 10
  ): Promise<PaginatedResponse<PaymentHistory>> =>
    apiClient.get(`/memberships/payments?page=${page}&page_size=${pageSize}`),
}
