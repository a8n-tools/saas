import { useEffect } from 'react'
import { useMembershipStore } from '@/stores/membershipStore'
import { useAuthStore } from '@/stores/authStore'

export function useMembership() {
  const store = useMembershipStore()
  const { isAuthenticated, user } = useAuthStore()

  useEffect(() => {
    if (isAuthenticated) {
      store.fetchMembership()
    }
  }, [isAuthenticated])

  const startCheckout = async (priceId?: string) => {
    const checkoutUrl = await store.createCheckout(priceId)
    // Validate URL before redirecting to Stripe checkout
    if (!checkoutUrl.startsWith('https://checkout.stripe.com/')) {
      throw new Error('Invalid checkout URL')
    }
    window.location.href = checkoutUrl
  }

  const subscribe = async () => {
    await store.subscribe()
  }

  return {
    membership: store.membership,
    isLoading: store.isLoading,
    error: store.error,
    startCheckout,
    subscribe,
    cancel: store.cancelMembership,
    reactivate: store.reactivateMembership,
    clearError: store.clearError,
    isActive: store.membership?.status === 'active',
    isPastDue: store.membership?.status === 'past_due',
    isCanceled: store.membership?.status === 'canceled',
    willCancel: store.membership?.cancel_at_period_end ?? false,
    tier: user?.subscription_tier ?? null,
  }
}
