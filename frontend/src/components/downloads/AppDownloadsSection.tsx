import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { downloadsApi } from '@/api/downloads'
import type { AppDownloadsResponse } from '@/types'
import { toast } from '@/components/ui/use-toast'

async function handleDownloadClick(e: React.MouseEvent, url: string) {
  e.preventDefault()
  try {
    const res = await fetch(url, { method: 'HEAD', credentials: 'include' })
    if (res.ok) {
      window.location.href = url
      return
    }
    if (res.status === 429) {
      const code = res.headers.get('x-error-code') ?? ''
      if (code.includes('concurrency')) {
        toast('You already have downloads in progress, please wait.')
      } else {
        const retry = Number(res.headers.get('retry-after') ?? 0)
        const hours = Math.ceil(retry / 3600)
        toast(`Daily download limit reached. Try again in ~${hours}h.`)
      }
      return
    }
    if (res.status === 502) {
      toast('Download source unavailable. Please try again later.')
      return
    }
    toast('Download failed.')
  } catch {
    toast('Download failed.')
  }
}

interface Props {
  slug: string
  hasMembership: boolean
}

function formatSize(bytes: number): string {
  const mb = bytes / 1_048_576
  if (mb >= 1) return `${mb.toFixed(1)} MB`
  return `${(bytes / 1024).toFixed(1)} KB`
}

export function AppDownloadsSection({ slug, hasMembership }: Props) {
  const [data, setData] = useState<AppDownloadsResponse | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    downloadsApi.listForApp(slug)
      .then(setData)
      .catch((e) => setError(e.message ?? 'Failed to load downloads'))
  }, [slug])

  if (error) {
    return <div className="text-sm text-destructive">{error}</div>
  }

  return (
    <section className="mt-6">
      <h2 className="text-lg font-semibold mb-2">Downloads</h2>
      {!data ? (
        <div className="text-sm text-muted-foreground">Loading…</div>
      ) : data.assets.length === 0 ? (
        <div className="text-sm text-muted-foreground">No downloads available</div>
      ) : (
        <div>
          <div className="text-xs text-muted-foreground mb-2">Release: {data.release_tag}</div>
          <ul className="space-y-2">
            {data.assets.map((asset) => (
              <li key={asset.asset_name} className="flex items-center justify-between border rounded p-3">
                <div>
                  <div className="font-mono text-sm">{asset.asset_name}</div>
                  <div className="text-xs text-muted-foreground">{formatSize(asset.size_bytes)}</div>
                </div>
                {hasMembership ? (
                  <a
                    href={asset.download_url}
                    className="inline-flex items-center px-3 py-1.5 rounded bg-primary text-primary-foreground text-sm"
                    onClick={(e) => handleDownloadClick(e, asset.download_url)}
                  >
                    Download
                  </a>
                ) : (
                  <Link to="/membership" className="text-sm text-primary underline">
                    Upgrade to access
                  </Link>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  )
}
