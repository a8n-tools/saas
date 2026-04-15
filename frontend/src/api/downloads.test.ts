import { describe, it, expect, vi, beforeEach } from 'vitest'
import { downloadsApi } from './downloads'
import { apiClient } from './client'

vi.mock('./client', () => ({
  apiClient: {
    get: vi.fn(),
    post: vi.fn(),
  },
}))

describe('downloadsApi', () => {
  beforeEach(() => vi.clearAllMocks())

  it('listForApp calls /applications/{slug}/downloads', async () => {
    ;(apiClient.get as any).mockResolvedValue({ release_tag: 'v1', assets: [] })
    const res = await downloadsApi.listForApp('rus')
    expect(apiClient.get).toHaveBeenCalledWith('/applications/rus/downloads')
    expect(res.release_tag).toBe('v1')
  })

  it('listAll calls /downloads', async () => {
    ;(apiClient.get as any).mockResolvedValue({ groups: [] })
    const res = await downloadsApi.listAll()
    expect(apiClient.get).toHaveBeenCalledWith('/downloads')
    expect(res).toEqual([])
  })

  it('adminRefresh calls admin refresh endpoint', async () => {
    ;(apiClient.post as any).mockResolvedValue({ release_tag: 'v2', assets: [] })
    const res = await downloadsApi.adminRefresh('rus')
    expect(apiClient.post).toHaveBeenCalledWith('/admin/applications/rus/downloads/refresh')
    expect(res.release_tag).toBe('v2')
  })
})
