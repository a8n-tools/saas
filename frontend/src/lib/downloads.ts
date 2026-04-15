import { toast } from '@/components/ui/use-toast'

export async function triggerDownload(url: string, filename: string): Promise<void> {
  try {
    const res = await fetch(url, { credentials: 'include' })
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
    if (!res.ok) {
      toast('Download failed.')
      return
    }
    const blob = await res.blob()
    const objectUrl = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = objectUrl
    a.download = filename
    document.body.appendChild(a)
    a.click()
    a.remove()
    URL.revokeObjectURL(objectUrl)
  } catch {
    toast('Download failed.')
  }
}
