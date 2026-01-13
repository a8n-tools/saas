import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { formatDate, formatCurrency } from '@/lib/utils'
import { CreditCard, MoreVertical, Loader2, XCircle, Gift } from 'lucide-react'
import { adminApi, type AdminSubscription } from '@/api/admin'

export function AdminSubscriptionsPage() {
  const [page, setPage] = useState(1)
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [selectedSub, setSelectedSub] = useState<AdminSubscription | null>(null)
  const [dialogType, setDialogType] = useState<'revoke' | null>(null)

  const queryClient = useQueryClient()

  const { data, isLoading } = useQuery({
    queryKey: ['admin', 'subscriptions', page, statusFilter],
    queryFn: () => adminApi.getSubscriptions(page, 20, statusFilter === 'all' ? undefined : statusFilter),
  })

  const { data: stats } = useQuery({
    queryKey: ['admin', 'stats'],
    queryFn: adminApi.getStats,
  })

  const revokeMutation = useMutation({
    mutationFn: (userId: string) => adminApi.revokeSubscription({ user_id: userId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'subscriptions'] })
      queryClient.invalidateQueries({ queryKey: ['admin', 'stats'] })
      setDialogType(null)
      setSelectedSub(null)
    },
  })

  const handleRevoke = (sub: AdminSubscription) => {
    setSelectedSub(sub)
    setDialogType('revoke')
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Subscriptions</h1>
        <p className="mt-2 text-muted-foreground">
          View and manage all subscriptions.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Active</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {stats?.active_subscribers ?? 0}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Past Due</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-yellow-600">
              {stats?.past_due_subscribers ?? 0}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Grace Period</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-orange-600">
              {stats?.grace_period_subscribers ?? 0}
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>All Subscriptions</CardTitle>
              <CardDescription>Manage customer subscriptions.</CardDescription>
            </div>
            <Select value={statusFilter} onValueChange={(v) => { setStatusFilter(v); setPage(1); }}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="Filter by status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="active">Active</SelectItem>
                <SelectItem value="past_due">Past Due</SelectItem>
                <SelectItem value="canceled">Canceled</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin" />
            </div>
          ) : (
            <>
              <div className="space-y-4">
                {data?.items.map((sub) => (
                  <div
                    key={sub.id}
                    className="flex items-center justify-between py-4 border-b last:border-0"
                  >
                    <div className="flex items-center gap-4">
                      <div className="flex h-10 w-10 items-center justify-center rounded-full bg-muted">
                        <CreditCard className="h-5 w-5 text-muted-foreground" />
                      </div>
                      <div>
                        <p className="font-medium">{sub.user_email}</p>
                        <p className="text-sm text-muted-foreground">
                          {formatCurrency(sub.amount_cents)} / month
                          {sub.current_period_end && ` - Ends ${formatDate(sub.current_period_end)}`}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <Badge variant="outline" className="capitalize">
                        {sub.tier}
                      </Badge>
                      <SubscriptionBadge status={sub.status} />
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon">
                            <MoreVertical className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          {sub.status === 'active' && (
                            <DropdownMenuItem onClick={() => handleRevoke(sub)}>
                              <XCircle className="h-4 w-4 mr-2" />
                              Revoke Subscription
                            </DropdownMenuItem>
                          )}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </div>
                ))}
                {data?.items.length === 0 && (
                  <p className="text-center text-muted-foreground py-8">
                    No subscriptions found
                  </p>
                )}
              </div>

              {data && data.total_pages > 1 && (
                <div className="flex justify-center gap-2 mt-6">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage((p) => Math.max(1, p - 1))}
                    disabled={page === 1}
                  >
                    Previous
                  </Button>
                  <span className="flex items-center px-3 text-sm">
                    Page {page} of {data.total_pages}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setPage((p) => Math.min(data.total_pages, p + 1))}
                    disabled={page === data.total_pages}
                  >
                    Next
                  </Button>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>

      <Dialog open={dialogType === 'revoke'} onOpenChange={() => {
        setDialogType(null)
        setSelectedSub(null)
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Revoke Subscription</DialogTitle>
            <DialogDescription>
              Are you sure you want to revoke the subscription for {selectedSub?.user_email}?
              They will immediately lose access to all applications.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setDialogType(null)
              setSelectedSub(null)
            }}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => selectedSub && revokeMutation.mutate(selectedSub.user_id)}
              disabled={revokeMutation.isPending}
            >
              {revokeMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              Revoke
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function SubscriptionBadge({ status }: { status: string }) {
  switch (status) {
    case 'active':
      return <Badge variant="success">Active</Badge>
    case 'past_due':
      return <Badge variant="warning">Past Due</Badge>
    case 'canceled':
      return <Badge variant="destructive">Canceled</Badge>
    default:
      return <Badge variant="outline">Unknown</Badge>
  }
}
