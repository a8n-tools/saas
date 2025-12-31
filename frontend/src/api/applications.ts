import { apiClient } from './client'
import type { Application } from '@/types'

export const applicationApi = {
  list: (): Promise<Application[]> => apiClient.get('/applications'),

  get: (slug: string): Promise<Application> =>
    apiClient.get(`/applications/${slug}`),
}
