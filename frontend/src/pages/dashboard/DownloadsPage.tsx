import { useEffect, useState } from 'react'
import { downloadsApi } from '@/api/downloads'
import type { AppDownloadGroup } from '@/types'
import { useAuthStore } from '@/stores/authStore'
import { hasActiveMembership } from '@/lib/utils'
import { Link } from 'react-router-dom'
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

function formatSize(bytes: number): string {
  const mb = bytes / 1_048_576
  if (mb >= 1) return `${mb.toFixed(1)} MB`
  return `${(bytes / 1024).toFixed(1)} KB`
}

export function DownloadsPage() {
  const [groups, setGroups] = useState<AppDownloadGroup[] | null>(null)
  const user = useAuthStore((s) => s.user)
  const hasMembership = hasActiveMembership(user)

  useEffect(() => {
    downloadsApi.listAll().then(setGroups).catch(() => setGroups([]))
  }, [])

  if (groups === null) return <div className="p-6 text-sm">Loading…</div>
  if (groups.length === 0) {
    return <div className="p-6 text-sm text-muted-foreground">No downloads available</div>
  }

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-semibold">Downloads</h1>
      {groups.map((g) => (
        <section key={g.app_slug} className="border rounded p-4">
          <div className="flex items-center gap-2 mb-2">
            {g.icon_url && <img src={g.icon_url} alt="" className="w-6 h-6" />}
            <h2 className="font-semibold">{g.app_display_name}</h2>
            <span className="text-xs text-muted-foreground">{g.release_tag}</span>
          </div>
          <ul className="space-y-2">
            {g.assets.map((a) => (
              <li key={a.asset_name} className="flex items-center justify-between">
                <div>
                  <div className="font-mono text-sm">{a.asset_name}</div>
                  <div className="text-xs text-muted-foreground">{formatSize(a.size_bytes)}</div>
                </div>
                {hasMembership ? (
                  <a
                    href={a.download_url}
                    className="px-3 py-1 rounded bg-primary text-primary-foreground text-sm"
                    onClick={(e) => handleDownloadClick(e, a.download_url)}
                  >
                    Download
                  </a>
                ) : (
                  <Link to="/membership" className="text-sm text-primary underline">Upgrade to access</Link>
                )}
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  )
}
