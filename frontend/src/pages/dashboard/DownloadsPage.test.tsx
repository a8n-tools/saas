import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAuthUser } from '@/test/utils'
import { DownloadsPage } from './DownloadsPage'
import { downloadsApi } from '@/api/downloads'
import * as useToastModule from '@/components/ui/use-toast'

vi.mock('@/api/downloads')
vi.mock('@/components/ui/use-toast', () => ({ toast: vi.fn() }))

const singleGroup = [
  {
    app_slug: 'rus',
    app_display_name: 'RUS',
    icon_url: null,
    release_tag: 'v1',
    assets: [{ asset_name: 'rus.bin', size_bytes: 100, content_type: 'x', download_url: '/v1/rus' }],
  },
]

describe('DownloadsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setupAuthUser()
  })

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

  it('shows concurrency toast on 429 concurrency', async () => {
    const toastMock = vi.mocked(useToastModule.toast)
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 429,
      headers: { get: (h: string) => h.toLowerCase() === 'x-error-code' ? 'download_concurrency_limit' : null },
    }))
    ;(downloadsApi.listAll as any).mockResolvedValue(singleGroup)
    const user = userEvent.setup()
    render(<DownloadsPage />)
    await waitFor(() => expect(screen.getByText('rus.bin')).toBeInTheDocument())
    await user.click(screen.getByRole('link', { name: /download/i }))
    await waitFor(() => expect(toastMock).toHaveBeenCalledWith(expect.stringMatching(/downloads in progress/i)))
  })

  it('shows 502 toast when download source unavailable', async () => {
    const toastMock = vi.mocked(useToastModule.toast)
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 502,
      headers: { get: () => null },
    }))
    ;(downloadsApi.listAll as any).mockResolvedValue(singleGroup)
    const user = userEvent.setup()
    render(<DownloadsPage />)
    await waitFor(() => expect(screen.getByText('rus.bin')).toBeInTheDocument())
    await user.click(screen.getByRole('link', { name: /download/i }))
    await waitFor(() => expect(toastMock).toHaveBeenCalledWith(expect.stringMatching(/unavailable/i)))
  })
})
