import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '@/test/utils'
import { AppDownloadsSection } from './AppDownloadsSection'
import { downloadsApi } from '@/api/downloads'
import * as useToastModule from '@/components/ui/use-toast'

vi.mock('@/api/downloads')
vi.mock('@/components/ui/use-toast', () => ({ toast: vi.fn() }))

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

  it('shows concurrency toast on 429 concurrency', async () => {
    const toastMock = vi.mocked(useToastModule.toast)
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 429,
      headers: { get: (h: string) => h.toLowerCase() === 'x-error-code' ? 'download_concurrency_limit' : null },
    }))
    ;(downloadsApi.listForApp as any).mockResolvedValue({
      release_tag: 'v1',
      assets: [{ asset_name: 'x.bin', size_bytes: 1, content_type: 'x', download_url: '/v1/x' }],
    })
    const user = userEvent.setup()
    render(<AppDownloadsSection slug="rus" hasMembership={true} />)
    await waitFor(() => expect(screen.getByText('x.bin')).toBeInTheDocument())
    await user.click(screen.getByRole('link', { name: /download/i }))
    await waitFor(() => expect(toastMock).toHaveBeenCalledWith(expect.stringMatching(/downloads in progress/i)))
  })

  it('shows daily-cap toast on 429 daily cap', async () => {
    const toastMock = vi.mocked(useToastModule.toast)
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 429,
      headers: { get: (h: string) => {
        if (h.toLowerCase() === 'x-error-code') return 'download_daily_limit'
        if (h.toLowerCase() === 'retry-after') return '3600'
        return null
      }},
    }))
    ;(downloadsApi.listForApp as any).mockResolvedValue({
      release_tag: 'v1',
      assets: [{ asset_name: 'x.bin', size_bytes: 1, content_type: 'x', download_url: '/v1/x' }],
    })
    const user = userEvent.setup()
    render(<AppDownloadsSection slug="rus" hasMembership={true} />)
    await waitFor(() => expect(screen.getByText('x.bin')).toBeInTheDocument())
    await user.click(screen.getByRole('link', { name: /download/i }))
    await waitFor(() => expect(toastMock).toHaveBeenCalledWith(expect.stringMatching(/daily/i)))
  })
})
