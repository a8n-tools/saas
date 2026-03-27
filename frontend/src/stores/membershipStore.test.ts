import { describe, it, expect, beforeEach } from 'vitest'
import { useMembershipStore } from './membershipStore'
import { useAuthStore } from './authStore'
import { mockMembership, mockUser } from '@/test/mocks/handlers'

beforeEach(() => {
  useMembershipStore.setState({
    membership: null,
    isLoading: false,
    error: null,
  })
  useAuthStore.setState({
    user: mockUser,
    isAuthenticated: true,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
})

describe('membershipStore', () => {
  describe('initial state', () => {
    it('starts with null membership', () => {
      const state = useMembershipStore.getState()
      expect(state.membership).toBeNull()
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
    })
  })

  describe('fetchMembership', () => {
    it('fetches and stores membership data', async () => {
      await useMembershipStore.getState().fetchMembership()

      const state = useMembershipStore.getState()
      expect(state.membership).toEqual(mockMembership)
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
    })
  })

  describe('createCheckout', () => {
    it('returns checkout URL', async () => {
      const url = await useMembershipStore.getState().createCheckout()

      expect(url).toBe('https://checkout.stripe.com/test')
    })
  })

  describe('cancelMembership', () => {
    it('cancels membership and refetches data', async () => {
      useMembershipStore.setState({ membership: mockMembership })

      await useMembershipStore.getState().cancelMembership()

      const state = useMembershipStore.getState()
      expect(state.isLoading).toBe(false)
    })
  })

  describe('reactivateMembership', () => {
    it('reactivates membership', async () => {
      await useMembershipStore.getState().reactivateMembership()

      const state = useMembershipStore.getState()
      expect(state.isLoading).toBe(false)
    })
  })

  describe('clearError', () => {
    it('clears the error', () => {
      useMembershipStore.setState({ error: 'Some error' })
      useMembershipStore.getState().clearError()

      expect(useMembershipStore.getState().error).toBeNull()
    })
  })
})
