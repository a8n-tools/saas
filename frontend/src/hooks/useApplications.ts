import { useQuery } from '@tanstack/react-query'
import { applicationApi } from '@/api'
import { useAuthStore } from '@/stores/authStore'
import { hasActiveMembership } from '@/lib/utils'

export function useApplications() {
  const user = useAuthStore((s) => s.user)
  const isMember = hasActiveMembership(user)

  const query = useQuery({
    queryKey: ['applications', { isMember }],
    queryFn: () => applicationApi.list(),
  })

  return {
    applications: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    refetch: query.refetch,
  }
}

export function useApplication(slug: string) {
  const query = useQuery({
    queryKey: ['application', slug],
    queryFn: () => applicationApi.get(slug),
    enabled: !!slug,
  })

  return {
    application: query.data,
    isLoading: query.isLoading,
    error: query.error,
  }
}
