import { useQuery } from '@tanstack/react-query'
import { applicationApi } from '@/api'

export function useApplications() {
  const query = useQuery({
    queryKey: ['applications'],
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
