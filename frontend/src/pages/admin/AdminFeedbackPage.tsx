import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { MessageSquareQuote, Loader2, AlertCircle, Mail, ArrowRight, Bug, Lightbulb, Sparkles, Workflow } from 'lucide-react'
import { adminApi, type AdminFeedbackDetail, type AdminFeedbackStatus } from '@/api/admin'
import { formatDate, formatRelativeTime } from '@/lib/utils'
import type { ApiError } from '@/types'
import { cn } from '@/lib/utils'

const statusVariant: Record<AdminFeedbackStatus, 'default' | 'secondary' | 'success' | 'warning'> = {
  new: 'warning',
  reviewed: 'secondary',
  responded: 'success',
  closed: 'default',
}

const feedbackTagStyles: Record<string, { icon: typeof Bug; className: string }> = {
  Bug: {
    icon: Bug,
    className: 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300',
  },
  Feature: {
    icon: Sparkles,
    className: 'border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-300',
  },
  Flow: {
    icon: Workflow,
    className: 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300',
  },
  Idea: {
    icon: Lightbulb,
    className: 'border-teal-500/30 bg-teal-500/10 text-teal-700 dark:text-teal-300',
  },
}

function formatPagePath(path: string) {
  if (path === '/') return 'Landing page'
  const stripped = path.replace(/^\//, '')
  return stripped.charAt(0).toUpperCase() + stripped.slice(1)
}

export function AdminFeedbackPage() {
  const queryClient = useQueryClient()
  const [searchParams, setSearchParams] = useSearchParams()
  const [page, setPage] = useState(1)
  const [selectedStatus, setSelectedStatus] = useState<AdminFeedbackStatus | 'all'>('all')
  const [selectedFeedback, setSelectedFeedback] = useState<AdminFeedbackDetail | null>(null)
  const [response, setResponse] = useState('')

  const linkedId = searchParams.get('id')

  useEffect(() => {
    if (!linkedId) return
    adminApi.getFeedbackDetail(linkedId).then((detail) => {
      setSelectedFeedback(detail)
      setResponse('')
    }).catch(() => {})
  }, [linkedId])

  const listQuery = useQuery({
    queryKey: ['admin', 'feedback', page, selectedStatus],
    queryFn: () => adminApi.getFeedback(page, 20, selectedStatus === 'all' ? undefined : selectedStatus),
  })

  const detailQuery = useQuery({
    queryKey: ['admin', 'feedback', selectedFeedback?.id],
    queryFn: () => adminApi.getFeedbackDetail(selectedFeedback!.id),
    enabled: !!selectedFeedback?.id,
  })

  const statusMutation = useMutation({
    mutationFn: ({ feedbackId, status }: { feedbackId: string; status: AdminFeedbackStatus }) =>
      adminApi.updateFeedbackStatus(feedbackId, status),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'feedback'] })
      setSelectedFeedback(data)
    },
  })

  // Auto-mark as reviewed after 5 seconds of viewing a new item
  useEffect(() => {
    const feedback = detailQuery.data ?? selectedFeedback
    if (!feedback || feedback.status !== 'new') return
    const timer = setTimeout(() => {
      statusMutation.mutate({ feedbackId: feedback.id, status: 'reviewed' })
    }, 5000)
    return () => clearTimeout(timer)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedFeedback?.id, detailQuery.data?.status])

  const respondMutation = useMutation({
    mutationFn: (feedbackId: string) => adminApi.respondToFeedback(feedbackId, {
      response,
      status: 'responded',
    }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'feedback'] })
      setSelectedFeedback(data)
      setResponse(data.admin_response ?? '')
    },
  })

  const activeFeedback = detailQuery.data ?? selectedFeedback

  const statusOptions = useMemo(
    () => ['all', 'new', 'reviewed', 'responded', 'closed'] as const,
    []
  )

  const openFeedback = (feedbackId: string) => {
    const summary = listQuery.data?.items.find((item) => item.id === feedbackId)
    if (summary) {
      setSelectedFeedback({
        id: summary.id,
        name: summary.name,
        email: null,
        email_masked: summary.email_masked,
        subject: summary.subject,
        tags: summary.tags,
        message: summary.message_excerpt,
        page_path: null,
        status: summary.status,
        admin_response: null,
        responded_by: null,
        responded_at: summary.responded_at,
        created_at: summary.created_at,
        updated_at: summary.created_at,
      })
      setResponse('')
    }
  }

  const renderTag = (tag: string) => {
    const config = feedbackTagStyles[tag]
    const TagIcon = config?.icon

    return (
      <Badge
        key={tag}
        variant="outline"
        className={cn('inline-flex items-center gap-1.5 bg-background/70', config?.className)}
      >
        {TagIcon && <TagIcon className="h-3.5 w-3.5" />}
        {tag}
      </Badge>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
        <div>
          <h1 className="text-3xl font-bold">Feedback</h1>
          <p className="mt-2 text-muted-foreground">
            Review what users are asking for and respond without leaving admin.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {statusOptions.map((status) => (
            <Button
              key={status}
              size="sm"
              variant={selectedStatus === status ? 'default' : 'outline'}
              onClick={() => {
                setSelectedStatus(status)
                setPage(1)
              }}
            >
              {status === 'all' ? 'All' : status}
            </Button>
          ))}
        </div>
      </div>

      {listQuery.isError && (
          <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Failed to load feedback</AlertTitle>
          <AlertDescription>{(listQuery.error as unknown as ApiError)?.error?.message || 'Please try again later.'}</AlertDescription>
        </Alert>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Incoming feedback</CardTitle>
          <CardDescription>Newest first. Email addresses stay masked here until you open an item.</CardDescription>
        </CardHeader>
        <CardContent>
          {listQuery.isLoading ? (
            <div className="flex justify-center py-10">
              <Loader2 className="h-8 w-8 animate-spin" />
            </div>
          ) : (
            <>
              <div className="space-y-4">
                {listQuery.data?.items.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => openFeedback(item.id)}
                    className="w-full rounded-xl border border-border/60 bg-background/50 p-4 text-left transition hover:border-primary/40 hover:bg-primary/5"
                  >
                    <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
                      <div className="space-y-2">
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge variant={statusVariant[item.status]}>{item.status}</Badge>
                          {item.subject && <span className="font-medium">{item.subject}</span>}
                        </div>
                        {item.tags.length > 0 && (
                          <div className="flex flex-wrap gap-2">
                            {item.tags.map(renderTag)}
                          </div>
                        )}
                        <p className="text-sm text-muted-foreground">{item.message_excerpt}</p>
                        <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                          {item.name && <span>{item.name}</span>}
                          {item.email_masked && (
                            <span className="inline-flex items-center gap-1">
                              <Mail className="h-3.5 w-3.5" />
                              {item.email_masked}
                            </span>
                          )}
                          <span>{formatRelativeTime(item.created_at)}</span>
                        </div>
                      </div>
                      <span className="inline-flex items-center gap-1 text-sm font-medium text-primary">
                        Open
                        <ArrowRight className="h-4 w-4" />
                      </span>
                    </div>
                  </button>
                ))}
              </div>

              {listQuery.data?.items.length === 0 && (
                <p className="py-10 text-center text-muted-foreground">No feedback found for this filter.</p>
              )}

              {listQuery.data && listQuery.data.total_pages > 1 && (
                <div className="mt-6 flex justify-center gap-2">
                  <Button variant="outline" size="sm" onClick={() => setPage((current) => Math.max(1, current - 1))} disabled={page === 1}>
                    Previous
                  </Button>
                  <span className="flex items-center px-3 text-sm">
                    Page {page} of {listQuery.data.total_pages}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage((current) => Math.min(listQuery.data.total_pages, current + 1))}
                    disabled={page === listQuery.data.total_pages}
                  >
                    Next
                  </Button>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>

      <Dialog open={!!selectedFeedback} onOpenChange={(open) => { if (!open) { setSelectedFeedback(null); setSearchParams({}, { replace: true }) } }}>
        <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <MessageSquareQuote className="h-5 w-5 text-primary" />
              Feedback detail
            </DialogTitle>
            <DialogDescription>
              Review the message, then save a response directly from admin.
            </DialogDescription>
          </DialogHeader>

          {!activeFeedback || detailQuery.isLoading ? (
            <div className="flex justify-center py-10">
              <Loader2 className="h-8 w-8 animate-spin" />
            </div>
          ) : (
            <div className="space-y-6">
              <div className="rounded-xl border border-border/60 bg-muted/30 p-4">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant={statusVariant[activeFeedback.status]}>{activeFeedback.status}</Badge>
                  {activeFeedback.subject && <span className="font-medium">{activeFeedback.subject}</span>}
                </div>
                {activeFeedback.tags.length > 0 && (
                  <div className="mt-3 flex flex-wrap gap-2">
                    {activeFeedback.tags.map(renderTag)}
                  </div>
                )}
                <div className="mt-3 grid gap-2 text-sm text-muted-foreground">
                  {activeFeedback.name && <p><span className="font-medium text-foreground">Name:</span> {activeFeedback.name}</p>}
                  {activeFeedback.email_masked && <p><span className="font-medium text-foreground">Email:</span> {activeFeedback.email_masked}</p>}
                  {activeFeedback.page_path && <p><span className="font-medium text-foreground">From:</span> {formatPagePath(activeFeedback.page_path)}</p>}
                  <p><span className="font-medium text-foreground">Created:</span> {formatDate(activeFeedback.created_at)}</p>
                </div>
              </div>

              <div className="grid gap-2">
                <Label>Message</Label>
                <div className="rounded-xl border border-border/60 bg-background/60 p-4">
                  <p className="whitespace-pre-wrap text-sm leading-6">{activeFeedback.message}</p>
                </div>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="admin-response">Response</Label>
                <Textarea
                  id="admin-response"
                  value={response}
                  onChange={(event) => setResponse(event.target.value)}
                  className="min-h-[160px] resize-y"
                  placeholder="Write the response that should be stored and emailed to the submitter."
                />
              </div>

              {activeFeedback.admin_response && !response && (
                <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/10 p-4 text-sm text-emerald-700 dark:text-emerald-300">
                  Existing response:
                  <p className="mt-2 whitespace-pre-wrap">{activeFeedback.admin_response}</p>
                </div>
              )}
            </div>
          )}

          <DialogFooter className="gap-2 sm:justify-between">
            <div className="flex gap-2">
              <Button variant="outline" onClick={() => setSelectedFeedback(null)}>
                Dismiss
              </Button>
              {activeFeedback && activeFeedback.status !== 'closed' && (
                <Button
                  variant="outline"
                  className="text-muted-foreground"
                  onClick={() => selectedFeedback && statusMutation.mutate({ feedbackId: selectedFeedback.id, status: 'closed' })}
                  disabled={statusMutation.isPending}
                >
                  {statusMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : 'Mark closed'}
                </Button>
              )}
            </div>
            <Button
              onClick={() => selectedFeedback && respondMutation.mutate(selectedFeedback.id)}
              disabled={!selectedFeedback || !response.trim() || respondMutation.isPending}
            >
              {respondMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Saving
                </>
              ) : (
                'Save response'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
