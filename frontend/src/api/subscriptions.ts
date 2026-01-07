import { apiClient } from './client'
import type {
  Subscription,
  PaymentHistory,
  CheckoutSessionResponse,
  CheckoutRequest,
  SubscriptionTier,
  PaginatedResponse,
} from '@/types'

export const subscriptionApi = {
  getCurrent: (): Promise<Subscription | null> =>
    apiClient.get('/v1/subscriptions/me'),

  createCheckout: (tier: SubscriptionTier = 'personal'): Promise<CheckoutSessionResponse> =>
    apiClient.post('/v1/subscriptions/checkout', { tier } as CheckoutRequest),

  cancel: (): Promise<Subscription> =>
    apiClient.post('/v1/subscriptions/cancel'),

  reactivate: (): Promise<Subscription> =>
    apiClient.post('/v1/subscriptions/reactivate'),

  getPaymentHistory: (
    page = 1,
    pageSize = 10
  ): Promise<PaginatedResponse<PaymentHistory>> =>
    apiClient.get(`/v1/subscriptions/payments?page=${page}&page_size=${pageSize}`),
}
