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
    apiClient.get('/subscriptions/me'),

  createCheckout: (tier: SubscriptionTier = 'personal'): Promise<CheckoutSessionResponse> =>
    apiClient.post('/subscriptions/checkout', { tier } as CheckoutRequest),

  cancel: (): Promise<Subscription> =>
    apiClient.post('/subscriptions/cancel'),

  reactivate: (): Promise<Subscription> =>
    apiClient.post('/subscriptions/reactivate'),

  getPaymentHistory: (
    page = 1,
    pageSize = 10
  ): Promise<PaginatedResponse<PaymentHistory>> =>
    apiClient.get(`/subscriptions/payments?page=${page}&page_size=${pageSize}`),
}
