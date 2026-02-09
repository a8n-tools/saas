import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/authStore'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
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
import { Search, MoreVertical, User, Loader2, KeyRound, Shield, ShieldOff, Trash2 } from 'lucide-react'
import { adminApi, type AdminUser } from '@/api/admin'
import { formatRelativeTime } from '@/lib/utils'

type DialogAction = 'deactivate' | 'activate' | 'reset' | 'delete' | 'makeAdmin' | 'removeAdmin' | null

export function AdminUsersPage() {
  const [searchQuery, setSearchQuery] = useState('')
  const [page, setPage] = useState(1)
  const [selectedUser, setSelectedUser] = useState<AdminUser | null>(null)
  const [dialogType, setDialogType] = useState<DialogAction>(null)
  const [tempPassword, setTempPassword] = useState<string | null>(null)

  const { user: currentUser } = useAuthStore()
  const queryClient = useQueryClient()

  const { data, isLoading } = useQuery({
    queryKey: ['admin', 'users', page, searchQuery],
    queryFn: () => adminApi.getUsers(page, 20, searchQuery || undefined),
  })

  const updateStatusMutation = useMutation({
    mutationFn: ({ userId, isActive }: { userId: string; isActive: boolean }) =>
      adminApi.updateUserStatus(userId, { is_active: isActive }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'users'] })
      setDialogType(null)
      setSelectedUser(null)
    },
  })

  const resetPasswordMutation = useMutation({
    mutationFn: (userId: string) => adminApi.resetUserPassword(userId),
    onSuccess: (data) => {
      setTempPassword(data.temporary_password)
    },
  })

  const deleteUserMutation = useMutation({
    mutationFn: (userId: string) => adminApi.deleteUser(userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'users'] })
      setDialogType(null)
      setSelectedUser(null)
    },
  })

  const updateRoleMutation = useMutation({
    mutationFn: ({ userId, role }: { userId: string; role: 'subscriber' | 'admin' }) =>
      adminApi.updateUserRole(userId, { role }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'users'] })
      setDialogType(null)
      setSelectedUser(null)
    },
  })

  const handleAction = (user: AdminUser, action: DialogAction) => {
    setSelectedUser(user)
    setDialogType(action)
    setTempPassword(null)
  }

  const confirmAction = () => {
    if (!selectedUser) return

    if (dialogType === 'deactivate') {
      updateStatusMutation.mutate({ userId: selectedUser.id, isActive: false })
    } else if (dialogType === 'activate') {
      updateStatusMutation.mutate({ userId: selectedUser.id, isActive: true })
    } else if (dialogType === 'reset') {
      resetPasswordMutation.mutate(selectedUser.id)
    } else if (dialogType === 'delete') {
      deleteUserMutation.mutate(selectedUser.id)
    } else if (dialogType === 'makeAdmin') {
      updateRoleMutation.mutate({ userId: selectedUser.id, role: 'admin' })
    } else if (dialogType === 'removeAdmin') {
      updateRoleMutation.mutate({ userId: selectedUser.id, role: 'subscriber' })
    }
  }

  const isActionPending = updateStatusMutation.isPending ||
    resetPasswordMutation.isPending ||
    deleteUserMutation.isPending ||
    updateRoleMutation.isPending

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Users</h1>
          <p className="mt-2 text-muted-foreground">
            Manage user accounts and memberships.
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-4">
            <div className="relative flex-1 max-w-sm">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search users..."
                value={searchQuery}
                onChange={(e) => {
                  setSearchQuery(e.target.value)
                  setPage(1)
                }}
                className="pl-9"
              />
            </div>
            {data && (
              <span className="text-sm text-muted-foreground">
                {data.total} users total
              </span>
            )}
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
                {data?.items.map((user) => (
                  <div
                    key={user.id}
                    className="flex items-center justify-between py-4 border-b last:border-0"
                  >
                    <div className="flex items-center gap-4">
                      <div className="flex h-10 w-10 items-center justify-center rounded-full bg-muted">
                        <User className="h-5 w-5 text-muted-foreground" />
                      </div>
                      <div>
                        <p className="font-medium">{user.email}</p>
                        <p className="text-sm text-muted-foreground">
                          Joined {formatRelativeTime(user.created_at)}
                          {user.last_login_at && ` - Last login ${formatRelativeTime(user.last_login_at)}`}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <Badge variant={user.role === 'admin' ? 'default' : 'secondary'}>
                        {user.role}
                      </Badge>
                      <MembershipBadge status={user.membership_status} />
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon">
                            <MoreVertical className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => handleAction(user, 'reset')}>
                            <KeyRound className="h-4 w-4 mr-2" />
                            Reset Password
                          </DropdownMenuItem>
                          {currentUser?.id !== user.id && (
                            <>
                              <DropdownMenuSeparator />
                              {user.role === 'admin' ? (
                                <DropdownMenuItem onClick={() => handleAction(user, 'removeAdmin')}>
                                  <ShieldOff className="h-4 w-4 mr-2" />
                                  Remove Admin
                                </DropdownMenuItem>
                              ) : (
                                <DropdownMenuItem onClick={() => handleAction(user, 'makeAdmin')}>
                                  <Shield className="h-4 w-4 mr-2" />
                                  Make Admin
                                </DropdownMenuItem>
                              )}
                              {user.role !== 'admin' && (
                                <>
                                  <DropdownMenuSeparator />
                                  <DropdownMenuItem
                                    onClick={() => handleAction(user, 'delete')}
                                    className="text-destructive focus:text-destructive"
                                  >
                                    <Trash2 className="h-4 w-4 mr-2" />
                                    Delete User
                                  </DropdownMenuItem>
                                </>
                              )}
                            </>
                          )}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </div>
                ))}
                {data?.items.length === 0 && (
                  <p className="text-center text-muted-foreground py-8">
                    No users found
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

      <Dialog open={dialogType !== null} onOpenChange={() => {
        setDialogType(null)
        setSelectedUser(null)
        setTempPassword(null)
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {dialogType === 'deactivate' && 'Deactivate User'}
              {dialogType === 'activate' && 'Activate User'}
              {dialogType === 'reset' && 'Reset Password'}
              {dialogType === 'delete' && 'Delete User'}
              {dialogType === 'makeAdmin' && 'Make Admin'}
              {dialogType === 'removeAdmin' && 'Remove Admin'}
            </DialogTitle>
            <DialogDescription>
              {dialogType === 'deactivate' && `Are you sure you want to deactivate ${selectedUser?.email}? They will lose access to all applications.`}
              {dialogType === 'activate' && `Are you sure you want to activate ${selectedUser?.email}?`}
              {dialogType === 'reset' && (
                tempPassword
                  ? 'Password has been reset. Share this temporary password with the user:'
                  : `Are you sure you want to reset the password for ${selectedUser?.email}?`
              )}
              {dialogType === 'delete' && `Are you sure you want to delete ${selectedUser?.email}? This action cannot be undone.`}
              {dialogType === 'makeAdmin' && `Are you sure you want to make ${selectedUser?.email} an admin? They will have full access to the admin panel.`}
              {dialogType === 'removeAdmin' && `Are you sure you want to remove admin privileges from ${selectedUser?.email}?`}
            </DialogDescription>
          </DialogHeader>

          {tempPassword && (
            <div className="bg-muted p-4 rounded-md font-mono text-center text-lg">
              {tempPassword}
            </div>
          )}

          <DialogFooter>
            {tempPassword ? (
              <Button onClick={() => {
                setDialogType(null)
                setSelectedUser(null)
                setTempPassword(null)
              }}>
                Done
              </Button>
            ) : (
              <>
                <Button variant="outline" onClick={() => {
                  setDialogType(null)
                  setSelectedUser(null)
                }}>
                  Cancel
                </Button>
                <Button
                  variant={dialogType === 'deactivate' || dialogType === 'delete' || dialogType === 'removeAdmin' ? 'destructive' : 'default'}
                  onClick={confirmAction}
                  disabled={isActionPending}
                >
                  {isActionPending && (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  )}
                  Confirm
                </Button>
              </>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function MembershipBadge({ status }: { status: string }) {
  switch (status) {
    case 'active':
      return <Badge variant="success">Active</Badge>
    case 'past_due':
      return <Badge variant="warning">Past Due</Badge>
    case 'canceled':
      return <Badge variant="destructive">Canceled</Badge>
    default:
      return <Badge variant="outline">None</Badge>
  }
}
