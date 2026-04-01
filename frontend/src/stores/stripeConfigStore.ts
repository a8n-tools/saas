import { create } from 'zustand'

interface StripeConfigState {
  stripeEnabled: boolean
  setStripeEnabled: (enabled: boolean) => void
}

export const useStripeConfigStore = create<StripeConfigState>((set) => ({
  stripeEnabled: true,
  setStripeEnabled: (enabled) => set({ stripeEnabled: enabled }),
}))
