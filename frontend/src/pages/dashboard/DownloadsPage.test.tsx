import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render } from '@/test/utils'
import { DownloadsPage } from './DownloadsPage'
import { downloadsApi } from '@/api/downloads'

vi.mock('@/api/downloads')

describe('DownloadsPage', () => {
  beforeEach(() => vi.clearAllMocks())

  it('renders an asset from each group', async () => {
    ;(downloadsApi.listAll as any).mockResolvedValue([
      {
        app_slug: 'rus',
        app_display_name: 'RUS',
        icon_url: null,
        release_tag: 'v1',
        assets: [{ asset_name: 'rus.bin', size_bytes: 100, content_type: 'x', download_url: '/v1/rus' }],
      },
    ])
    render(<DownloadsPage />)
    await waitFor(() => expect(screen.getByText('RUS')).toBeInTheDocument())
    expect(screen.getByText('rus.bin')).toBeInTheDocument()
  })

  it('renders empty state when no groups', async () => {
    ;(downloadsApi.listAll as any).mockResolvedValue([])
    render(<DownloadsPage />)
    await waitFor(() => expect(screen.getByText(/No downloads available/i)).toBeInTheDocument())
  })
})
