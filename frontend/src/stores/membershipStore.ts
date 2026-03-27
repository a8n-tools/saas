import { create } from 'zustand'
import type { Membership } from '@/types'
import { membershipApi } from '@/api'
import { useAuthStore } from './authStore'

interface MembershipState {
  membership: Membership | null
  isLoading: boolean
  error: string | null

  // Actions
  fetchMembership: () => Promise<void>
  createCheckout: (priceId?: string) => Promise<string>
  subscribe: () => Promise<void>
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

  createCheckout: async (priceId?: string) => {
    set({ isLoading: true, error: null })
    try {
      const response = await membershipApi.createCheckout(priceId)
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

  subscribe: async () => {
    set({ isLoading: true, error: null })
    try {
      await membershipApi.subscribe()
      // Refetch membership to get updated state
      const membership = await membershipApi.getCurrent()
      set({ membership, isLoading: false })
      // Also refresh user data since JWT claims changed
      await useAuthStore.getState().refreshUser()
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to subscribe',
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
      // Also refresh user data since JWT claims changed
      await useAuthStore.getState().refreshUser()
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
