import { create } from 'zustand'
import type { Membership, MembershipTier } from '@/types'
import { membershipApi } from '@/api'

interface MembershipState {
  membership: Membership | null
  isLoading: boolean
  error: string | null

  // Actions
  fetchMembership: () => Promise<void>
  createCheckout: (tier?: MembershipTier) => Promise<string>
  cancelMembership: () => Promise<void>
  reactivateMembership: () => Promise<void>
  clearError: () => void
}

export const useMembershipStore = create<MembershipState>((set) => ({
  membership: null,
  isLoading: false,
  error: null,

  fetchMembership: async () => {
    set({ isLoading: true, error: null })
    try {
      const membership = await membershipApi.getCurrent()
      set({ membership, isLoading: false })
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to fetch membership',
        isLoading: false,
      })
    }
  },

  createCheckout: async (tier: MembershipTier = 'personal') => {
    set({ isLoading: true, error: null })
    try {
      const response = await membershipApi.createCheckout(tier)
      set({ isLoading: false })
      return response.checkout_url
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to create checkout',
        isLoading: false,
      })
      throw err
    }
  },

  cancelMembership: async () => {
    set({ isLoading: true, error: null })
    try {
      await membershipApi.cancel()
      // Refetch membership to get updated state
      const membership = await membershipApi.getCurrent()
      set({ membership, isLoading: false })
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to cancel membership',
        isLoading: false,
      })
      throw err
    }
  },

  reactivateMembership: async () => {
    set({ isLoading: true, error: null })
    try {
      await membershipApi.reactivate()
      // Refetch membership to get updated state
      const membership = await membershipApi.getCurrent()
      set({ membership, isLoading: false })
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to reactivate membership',
        isLoading: false,
      })
      throw err
    }
  },

  clearError: () => set({ error: null }),
}))
