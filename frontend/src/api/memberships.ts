import { apiClient } from './client'
import type {
  Membership,
  StripePaymentResponse,
  CheckoutSessionResponse,
} from '@/types'

export interface SubscribeResponse {
  message: string
  membership_status: string
}

export const membershipApi = {
  getCurrent: (): Promise<Membership | null> =>
    apiClient.get('/memberships/me'),

  createCheckout: (priceId?: string): Promise<CheckoutSessionResponse> =>
    apiClient.post('/memberships/checkout', priceId ? { price_id: priceId } : {}),

  subscribe: (): Promise<SubscribeResponse> =>
    apiClient.post('/memberships/subscribe'),

  cancel: (): Promise<void> =>
    apiClient.post('/memberships/cancel'),

  cancelNow: (): Promise<void> =>
    apiClient.post('/memberships/cancel-now'),

  reactivate: (): Promise<void> =>
    apiClient.post('/memberships/reactivate'),

  getPaymentHistory: (): Promise<StripePaymentResponse[]> =>
    apiClient.get('/memberships/payments'),
}
