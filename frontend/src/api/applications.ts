import { apiClient } from './client'
import type { Application } from '@/types'

export const applicationApi = {
  list: async (): Promise<Application[]> => {
    const response = await apiClient.get<{ applications: Application[] }>('/applications')
    return response.applications
  },

  get: (slug: string): Promise<Application> =>
    apiClient.get(`/applications/${slug}`),
}
