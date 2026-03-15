import { describe, it, expect } from 'vitest'
import { membershipApi } from './memberships'
import { mockMembership } from '@/test/mocks/handlers'

describe('membershipApi', () => {
  describe('getCurrent', () => {
    it('returns current membership', async () => {
      const membership = await membershipApi.getCurrent()

      expect(membership).toEqual(mockMembership)
    })
  })

  describe('createCheckout', () => {
    it('returns checkout URL for personal tier', async () => {
      const response = await membershipApi.createCheckout('personal')

      expect(response.checkout_url).toBe('https://checkout.stripe.com/test')
      expect(response.session_id).toBe('cs_test')
    })

    it('returns checkout URL for business tier', async () => {
      const response = await membershipApi.createCheckout('business')

      expect(response.checkout_url).toBe('https://checkout.stripe.com/test')
    })
  })

  describe('cancel', () => {
    it('cancels membership without error', async () => {
      await expect(membershipApi.cancel()).resolves.not.toThrow()
    })
  })

  describe('reactivate', () => {
    it('reactivates membership without error', async () => {
      await expect(membershipApi.reactivate()).resolves.not.toThrow()
    })
  })

  describe('getPaymentHistory', () => {
    it('returns payment history', async () => {
      const history = await membershipApi.getPaymentHistory()

      expect(history.items).toEqual([])
      expect(history.total).toBe(0)
    })
  })
})
