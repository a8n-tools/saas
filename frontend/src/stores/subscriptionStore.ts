import { create } from 'zustand'
import type { Subscription, SubscriptionTier } from '@/types'
import { subscriptionApi } from '@/api'

interface SubscriptionState {
  subscription: Subscription | null
  isLoading: boolean
  error: string | null

  // Actions
  fetchSubscription: () => Promise<void>
  createCheckout: (tier?: SubscriptionTier) => Promise<string>
  cancelSubscription: () => Promise<void>
  reactivateSubscription: () => Promise<void>
  clearError: () => void
}

export const useSubscriptionStore = create<SubscriptionState>((set) => ({
  subscription: null,
  isLoading: false,
  error: null,

  fetchSubscription: async () => {
    set({ isLoading: true, error: null })
    try {
      const subscription = await subscriptionApi.getCurrent()
      set({ subscription, isLoading: false })
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to fetch subscription',
        isLoading: false,
      })
    }
  },

  createCheckout: async (tier: SubscriptionTier = 'personal') => {
    set({ isLoading: true, error: null })
    try {
      const response = await subscriptionApi.createCheckout(tier)
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

  cancelSubscription: async () => {
    set({ isLoading: true, error: null })
    try {
      const subscription = await subscriptionApi.cancel()
      set({ subscription, isLoading: false })
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to cancel subscription',
        isLoading: false,
      })
      throw err
    }
  },

  reactivateSubscription: async () => {
    set({ isLoading: true, error: null })
    try {
      const subscription = await subscriptionApi.reactivate()
      set({ subscription, isLoading: false })
    } catch (err) {
      const error = err as { error?: { message?: string } }
      set({
        error: error.error?.message || 'Failed to reactivate subscription',
        isLoading: false,
      })
      throw err
    }
  },

  clearError: () => set({ error: null }),
}))
