import { apiClient } from './client'
import type { Application } from '@/types'

export const applicationApi = {
  list: (): Promise<Application[]> => apiClient.get('/v1/applications'),

  get: (slug: string): Promise<Application> =>
    apiClient.get(`/v1/applications/${slug}`),
}
