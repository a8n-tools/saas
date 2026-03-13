import { useMemo, useRef, useState } from 'react'
import { useLocation } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import { MessageCircleHeart, Loader2, Send, AlertCircle, Bug, Lightbulb, Sparkles, Workflow, X, Paperclip } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { feedbackApi } from '@/api/feedback'
import type { ApiError } from '@/types'
import { cn } from '@/lib/utils'

const MAX_FILES = 3
const MAX_FILE_SIZE = 5 * 1024 * 1024
const ACCEPTED_FILE_TYPES = 'image/png,image/jpeg,image/webp,image/gif,text/plain'

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const feedbackTagOptions = [
  {
    label: 'Bug',
    icon: Bug,
    classes: {
      selected: 'border-rose-500/45 bg-rose-500/18 text-rose-700 hover:border-rose-500/60 hover:bg-rose-500/24 dark:text-rose-200 dark:hover:bg-rose-500/20',
      idle: 'border-rose-500/30 text-rose-600 hover:border-rose-500/50 hover:bg-rose-500/10 dark:text-rose-300 dark:hover:bg-rose-500/10',
      icon: 'text-current',
    },
  },
  {
    label: 'Feature',
    icon: Sparkles,
    classes: {
      selected: 'border-sky-500/45 bg-sky-500/18 text-sky-700 hover:border-sky-500/60 hover:bg-sky-500/24 dark:text-sky-200 dark:hover:bg-sky-500/20',
      idle: 'border-sky-500/30 text-sky-600 hover:border-sky-500/50 hover:bg-sky-500/10 dark:text-sky-300 dark:hover:bg-sky-500/10',
      icon: 'text-current',
    },
  },
  {
    label: 'Flow',
    icon: Workflow,
    classes: {
      selected: 'border-amber-500/45 bg-amber-500/18 text-amber-700 hover:border-amber-500/60 hover:bg-amber-500/24 dark:text-amber-200 dark:hover:bg-amber-500/20',
      idle: 'border-amber-500/30 text-amber-600 hover:border-amber-500/50 hover:bg-amber-500/10 dark:text-amber-300 dark:hover:bg-amber-500/10',
      icon: 'text-current',
    },
  },
  {
    label: 'Idea',
    icon: Lightbulb,
    classes: {
      selected: 'border-teal-500/45 bg-teal-500/18 text-teal-700 hover:border-teal-500/60 hover:bg-teal-500/24 dark:text-teal-200 dark:hover:bg-teal-500/20',
      idle: 'border-teal-500/30 text-teal-600 hover:border-teal-500/50 hover:bg-teal-500/10 dark:text-teal-300 dark:hover:bg-teal-500/10',
      icon: 'text-current',
    },
  },
] as const

interface FormState {
  name: string
  email: string
  subject: string
  tags: string[]
  message: string
  website: string
}

const initialState: FormState = {
  name: '',
  email: '',
  subject: '',
  tags: [],
  message: '',
  website: '',
}

export function FeedbackPage() {
  const location = useLocation()
  const [form, setForm] = useState<FormState>(initialState)
  const [files, setFiles] = useState<File[]>([])
  const [fileError, setFileError] = useState<string | null>(null)
  const [submitted, setSubmitted] = useState(false)
  const fileInputRef = useRef<HTMLInputElement | null>(null)

  const pagePath = useMemo(() => {
    const state = location.state as { fromPath?: string } | null
    return state?.fromPath || '/feedback'
  }, [location.state])

  const pagePathLabel = useMemo(() => {
    if (pagePath === '/') return 'Landing page'
    const stripped = pagePath.replace(/^\//, '')
    return stripped.charAt(0).toUpperCase() + stripped.slice(1)
  }, [pagePath])

  const submitMutation = useMutation({
    mutationFn: () => feedbackApi.submit({
      name: form.name || undefined,
      email: form.email || undefined,
      subject: form.subject || undefined,
      tags: form.tags,
      message: form.message,
      page_path: pagePath,
      website: form.website || undefined,
      files,
    }),
    onSuccess: () => {
      setSubmitted(true)
      setForm(initialState)
      setFiles([])
      setFileError(null)
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }
    },
  })

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault()
    setSubmitted(false)
    setFileError(null)
    submitMutation.mutate()
  }

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(event.target.files ?? [])
    if (selectedFiles.length === 0) return

    if (files.length + selectedFiles.length > MAX_FILES) {
      setFileError(`You can attach up to ${MAX_FILES} files.`)
      event.target.value = ''
      return
    }

    const oversized = selectedFiles.find((file) => file.size > MAX_FILE_SIZE)
    if (oversized) {
      setFileError(`${oversized.name} exceeds the 5 MB limit.`)
      event.target.value = ''
      return
    }

    setFiles((current) => [...current, ...selectedFiles])
    setFileError(null)
    event.target.value = ''
  }

  const removeFile = (index: number) => {
    setFiles((current) => current.filter((_, currentIndex) => currentIndex !== index))
    setFileError(null)
  }

  const toggleTag = (tag: string) => {
    setForm((current) => ({
      ...current,
      tags: current.tags.includes(tag)
        ? current.tags.filter((currentTag) => currentTag !== tag)
        : [...current.tags, tag],
    }))
  }

  return (
    <div className="relative overflow-hidden py-20">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top_right,rgba(249,115,22,0.16),transparent_35%),radial-gradient(circle_at_bottom_left,rgba(20,184,166,0.16),transparent_30%)]" />
      <div className="container relative">
        <div className="mx-auto max-w-3xl text-center">
          <div className="inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/10 px-4 py-1.5 text-sm text-primary">
            <MessageCircleHeart className="h-4 w-4" />
            Help shape what ships next
          </div>
          <h1 className="mt-6 text-4xl font-bold tracking-tight sm:text-5xl">
            Tell us what would make <span className="text-gradient bg-gradient-to-r from-primary via-indigo-500 to-teal-400">a8n.tools</span> better.
          </h1>
          <p className="mx-auto mt-4 max-w-2xl text-lg text-muted-foreground">
            Share bugs, missing features, rough edges, or ideas. We read everything, and we use it to decide what gets polished next.
          </p>
        </div>

        <Card className="mx-auto mt-12 max-w-2xl border-border/60 bg-card/90 shadow-xl shadow-primary/10 backdrop-blur-sm">
          <CardHeader>
            <CardTitle>Send feedback</CardTitle>
            <CardDescription>
              Leave your email if you would like a follow-up. We kept this form simple so it is easy to share what is on your mind.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {submitted && (
              <Alert className="mb-6 border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300">
                <MessageCircleHeart className="h-4 w-4" />
                <AlertTitle>Feedback sent</AlertTitle>
                <AlertDescription>Thanks for sharing. Your feedback has been sent to the team.</AlertDescription>
              </Alert>
            )}

            {submitMutation.isError && (
              <Alert variant="destructive" className="mb-6">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>Could not send feedback</AlertTitle>
                <AlertDescription>
                  {(submitMutation.error as unknown as ApiError)?.error?.message || 'Please try again in a moment.'}
                </AlertDescription>
              </Alert>
            )}

            <form className="space-y-5" onSubmit={handleSubmit}>
              <input
                ref={fileInputRef}
                type="file"
                hidden
                multiple
                accept={ACCEPTED_FILE_TYPES}
                onChange={handleFileChange}
              />

              <div className="grid gap-5 md:grid-cols-2">
                <div className="grid gap-2">
                  <Label htmlFor="name">Name</Label>
                  <Input
                    id="name"
                    value={form.name}
                    onChange={(e) => setForm((current) => ({ ...current, name: e.target.value }))}
                    placeholder="Optional"
                    maxLength={100}
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="email">Email</Label>
                  <Input
                    id="email"
                    type="email"
                    value={form.email}
                    onChange={(e) => setForm((current) => ({ ...current, email: e.target.value }))}
                    placeholder="you@example.com"
                    required
                    maxLength={255}
                  />
                </div>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="subject">Subject</Label>
                <Input
                  id="subject"
                  value={form.subject}
                  onChange={(e) => setForm((current) => ({ ...current, subject: e.target.value }))}
                  placeholder="Optional"
                  maxLength={200}
                />
              </div>

              <div className="grid gap-3">
                <div className="space-y-1">
                  <Label>Attach a few tags</Label>
                  <p className="text-sm text-muted-foreground">
                    Pick any that fit. They help us understand the kind of feedback you are sharing.
                  </p>
                </div>
                <div className="flex flex-wrap gap-2">
                  {feedbackTagOptions.map((tag) => {
                    const selected = form.tags.includes(tag.label)
                    const TagIcon = tag.icon
                    return (
                      <Button
                        key={tag.label}
                        type="button"
                        variant="outline"
                        size="sm"
                        className={cn(
                          'gap-2 rounded-full border bg-background/80 pr-3 transition-all',
                          selected ? tag.classes.selected : tag.classes.idle
                        )}
                        onClick={() => toggleTag(tag.label)}
                      >
                        <span className="inline-flex h-4 w-4 items-center justify-center">
                          <TagIcon className={cn('h-3.5 w-3.5', tag.classes.icon)} />
                        </span>
                        <span>{tag.label}</span>
                        {selected && (
                          <span className="ml-1 inline-flex h-4 w-4 items-center justify-center rounded-full border border-current/20 bg-background/25">
                            <X className="h-3 w-3" />
                          </span>
                        )}
                      </Button>
                    )
                  })}
                </div>
              </div>

              <div className="hidden">
                <Label htmlFor="website">Website</Label>
                <Input
                  id="website"
                  tabIndex={-1}
                  autoComplete="off"
                  value={form.website}
                  onChange={(e) => setForm((current) => ({ ...current, website: e.target.value }))}
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="message">Message</Label>
                <Textarea
                  id="message"
                  value={form.message}
                  onChange={(e) => setForm((current) => ({ ...current, message: e.target.value }))}
                  placeholder="What would you like to see improved?"
                  className="min-h-[180px] resize-y"
                  required
                  maxLength={5000}
                />
                <p className="text-xs text-muted-foreground">
                  Shared from: <span className="font-medium text-foreground">{pagePathLabel}</span>
                </p>
              </div>

              <div className="grid gap-3">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                  <div className="space-y-1">
                    <Label>Attachments</Label>
                    <p className="text-sm text-muted-foreground">
                      Up to 3 files, 5 MB each. Images and plain text only.
                    </p>
                  </div>
                  <Button
                    type="button"
                    variant="outline"
                    className="gap-2"
                    onClick={() => fileInputRef.current?.click()}
                  >
                    <Paperclip className="h-4 w-4" />
                    Attach files
                  </Button>
                </div>

                {files.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {files.map((file, index) => (
                      <div
                        key={`${file.name}-${file.size}-${index}`}
                        className="inline-flex items-center gap-2 rounded-full border border-border/70 bg-muted/40 px-3 py-1.5 text-sm"
                      >
                        <Paperclip className="h-3.5 w-3.5 text-muted-foreground" />
                        <span>{file.name}</span>
                        <span className="text-xs text-muted-foreground">({formatBytes(file.size)})</span>
                        <button
                          type="button"
                          onClick={() => removeFile(index)}
                          className="inline-flex text-muted-foreground transition hover:text-foreground"
                          aria-label={`Remove ${file.name}`}
                        >
                          <X className="h-3.5 w-3.5" />
                        </button>
                      </div>
                    ))}
                  </div>
                )}

                {fileError && (
                  <p className="text-sm text-destructive">{fileError}</p>
                )}
              </div>

              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <p className="text-sm text-muted-foreground">
                  Share as much or as little context as feels helpful.
                </p>
                <Button type="submit" className="gap-2" disabled={submitMutation.isPending}>
                  {submitMutation.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      Sending
                    </>
                  ) : (
                    <>
                      Send feedback
                      <Send className="h-4 w-4" />
                    </>
                  )}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
