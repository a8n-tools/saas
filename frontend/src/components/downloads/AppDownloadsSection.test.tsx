import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render } from '@/test/utils'
import { AppDownloadsSection } from './AppDownloadsSection'
import { downloadsApi } from '@/api/downloads'

vi.mock('@/api/downloads')

const sample = {
  release_tag: 'v1.2.3',
  assets: [
    { asset_name: 'rus-linux.tar.gz', size_bytes: 1048576, content_type: 'application/gzip', download_url: '/v1/applications/rus/downloads/rus-linux.tar.gz' },
  ],
}

describe('AppDownloadsSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('renders empty state when no assets', async () => {
    ;(downloadsApi.listForApp as any).mockResolvedValue({ release_tag: null, assets: [] })
    render(<AppDownloadsSection slug="rus" hasMembership={true} />)
    await waitFor(() => expect(screen.getByText(/No downloads available/i)).toBeInTheDocument())
  })

  it('renders assets with download buttons when member', async () => {
    ;(downloadsApi.listForApp as any).mockResolvedValue(sample)
    render(<AppDownloadsSection slug="rus" hasMembership={true} />)
    await waitFor(() => expect(screen.getByText('rus-linux.tar.gz')).toBeInTheDocument())
    expect(screen.getByText(/v1.2.3/)).toBeInTheDocument()
    const link = screen.getByRole('link', { name: /download/i })
    expect(link).toHaveAttribute('href', sample.assets[0].download_url)
  })

  it('renders gated CTA when not a member', async () => {
    ;(downloadsApi.listForApp as any).mockResolvedValue(sample)
    render(<AppDownloadsSection slug="rus" hasMembership={false} />)
    await waitFor(() => expect(screen.getByText('rus-linux.tar.gz')).toBeInTheDocument())
    expect(screen.queryByRole('link', { name: /download/i })).toBeNull()
    expect(screen.getByRole('link', { name: /upgrade/i })).toBeInTheDocument()
  })
})
