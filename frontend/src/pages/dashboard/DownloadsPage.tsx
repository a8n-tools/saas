import { useEffect, useState } from 'react'
import { downloadsApi } from '@/api/downloads'
import type { AppDownloadGroup } from '@/types'
import { useAuthStore } from '@/stores/authStore'
import { hasActiveMembership } from '@/lib/utils'
import { Link } from 'react-router-dom'
import { triggerDownload } from '@/lib/downloads'

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
                    download={a.asset_name}
                    aria-label={`Download ${a.asset_name}`}
                    className="px-3 py-1 rounded bg-primary text-primary-foreground text-sm"
                    onClick={(e) => {
                      e.preventDefault()
                      triggerDownload(a.download_url, a.asset_name)
                    }}
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
