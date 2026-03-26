import { apiClient } from './client'
import type { StripeInvoice } from '@/types'

export const billingApi = {
  listInvoices: (): Promise<StripeInvoice[]> =>
    apiClient.get('/billing/invoices'),
}
