import { useEffect } from 'react'
import { useMembershipStore } from '@/stores/membershipStore'
import { useAuthStore } from '@/stores/authStore'
import type { MembershipTier } from '@/types'

export function useMembership() {
  const store = useMembershipStore()
  const { isAuthenticated, user } = useAuthStore()

  useEffect(() => {
    if (isAuthenticated) {
      store.fetchMembership()
    }
  }, [isAuthenticated])

  const startCheckout = async (tier: MembershipTier = 'personal') => {
    const checkoutUrl = await store.createCheckout(tier)
    // Redirect to Stripe checkout
    window.location.href = checkoutUrl
  }

  return {
    membership: store.membership,
    isLoading: store.isLoading,
    error: store.error,
    startCheckout,
    cancel: store.cancelMembership,
    reactivate: store.reactivateMembership,
    clearError: store.clearError,
    isActive: store.membership?.status === 'active',
    isPastDue: store.membership?.status === 'past_due',
    isCanceled: store.membership?.status === 'canceled',
    willCancel: store.membership?.cancel_at_period_end ?? false,
    tier: user?.membership_tier ?? null,
  }
}
