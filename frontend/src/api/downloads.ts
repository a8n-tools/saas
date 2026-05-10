import { apiClient } from './client'
import type { AppDownloadsResponse, AppDownloadGroup } from '@/types'

export const downloadsApi = {
  listForApp: (slug: string): Promise<AppDownloadsResponse> =>
    apiClient.get<AppDownloadsResponse>(`/applications/${slug}/downloads`),

  listAll: async (): Promise<AppDownloadGroup[]> => {
    const res = await apiClient.get<{ groups: AppDownloadGroup[] }>('/downloads')
    return res.groups
  },

  adminRefresh: (slug: string): Promise<AppDownloadsResponse> =>
    apiClient.post<AppDownloadsResponse>(`/admin/applications/${slug}/downloads/refresh`),
}
