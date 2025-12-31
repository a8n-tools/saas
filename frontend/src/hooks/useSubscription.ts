import { useEffect } from 'react'
import { useSubscriptionStore } from '@/stores/subscriptionStore'
import { useAuthStore } from '@/stores/authStore'

export function useSubscription() {
  const store = useSubscriptionStore()
  const { isAuthenticated } = useAuthStore()

  useEffect(() => {
    if (isAuthenticated) {
      store.fetchSubscription()
    }
  }, [isAuthenticated])

  const startCheckout = async () => {
    const checkoutUrl = await store.createCheckout()
    // Redirect to Stripe checkout
    window.location.href = checkoutUrl
  }

  return {
    subscription: store.subscription,
    isLoading: store.isLoading,
    error: store.error,
    startCheckout,
    cancel: store.cancelSubscription,
    reactivate: store.reactivateSubscription,
    clearError: store.clearError,
    isActive: store.subscription?.status === 'active',
    isPastDue: store.subscription?.status === 'past_due',
    isCanceled: store.subscription?.status === 'canceled',
    willCancel: store.subscription?.cancel_at_period_end ?? false,
  }
}
