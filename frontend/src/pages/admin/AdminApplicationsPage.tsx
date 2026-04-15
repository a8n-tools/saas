import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Loader2, AppWindow, ExternalLink, Pencil, AlertCircle, Plus, Trash2, ShieldAlert, ArrowUp, ArrowDown, RefreshCw } from 'lucide-react'
import { adminApi } from '@/api/admin'
import { authApi } from '@/api/auth'
import { downloadsApi } from '@/api/downloads'
import type { AdminApplication, UpdateApplicationRequest, CreateApplicationRequest } from '@/api/admin'
import type { AppDownloadsResponse } from '@/types'
import { config } from '@/config'
import type { ApiError } from '@/types'

export function AdminApplicationsPage() {
  const queryClient = useQueryClient()
  const [editingApp, setEditingApp] = useState<AdminApplication | null>(null)
  const [editForm, setEditForm] = useState<UpdateApplicationRequest>({})
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [createForm, setCreateForm] = useState<CreateApplicationRequest>({
    name: '',
    slug: '',
    display_name: '',
    container_name: '',
  })
  const [deletingApp, setDeletingApp] = useState<AdminApplication | null>(null)
  const [deleteStep, setDeleteStep] = useState<1 | 2>(1)
  const [deletePassword, setDeletePassword] = useState('')
  const [deleteTotpCode, setDeleteTotpCode] = useState('')
  const [deleteError, setDeleteError] = useState('')
  const [refreshResult, setRefreshResult] = useState<AppDownloadsResponse | null>(null)
  const [refreshError, setRefreshError] = useState('')
  const [isRefreshing, setIsRefreshing] = useState(false)

  const { data: applications, isLoading, isError, error } = useQuery({
    queryKey: ['admin', 'applications'],
    queryFn: adminApi.getApplications,
  })

  const { data: twoFactorStatus } = useQuery({
    queryKey: ['2fa', 'status'],
    queryFn: authApi.get2FAStatus,
  })

  const updateMutation = useMutation({
    mutationFn: ({ appId, data }: { appId: string; data: UpdateApplicationRequest }) =>
      adminApi.updateApplication(appId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'applications'] })
      queryClient.invalidateQueries({ queryKey: ['applications'] })
    },
  })

  const editMutation = useMutation({
    mutationFn: ({ appId, data }: { appId: string; data: UpdateApplicationRequest }) =>
      adminApi.updateApplication(appId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'applications'] })
      queryClient.invalidateQueries({ queryKey: ['applications'] })
      setEditingApp(null)
    },
  })

  const createMutation = useMutation({
    mutationFn: (data: CreateApplicationRequest) => adminApi.createApplication(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'applications'] })
      queryClient.invalidateQueries({ queryKey: ['applications'] })
      setShowCreateDialog(false)
      setCreateForm({ name: '', slug: '', display_name: '', container_name: '' })
    },
  })

  const reorderMutation = useMutation({
    mutationFn: ({ appId, targetAppId }: { appId: string; targetAppId: string }) =>
      adminApi.swapApplicationOrder(appId, targetAppId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'applications'] })
      queryClient.invalidateQueries({ queryKey: ['applications'] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: ({ appId, password, totp_code }: { appId: string; password: string; totp_code: string }) =>
      adminApi.deleteApplication(appId, { password, totp_code }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'applications'] })
      queryClient.invalidateQueries({ queryKey: ['applications'] })
      closeDeleteDialog()
    },
    onError: (err: ApiError) => {
      setDeleteError(err?.error?.message || 'Failed to delete application')
    },
  })

  const handleToggleActive = (app: AdminApplication) => {
    updateMutation.mutate({
      appId: app.id,
      data: { is_active: !app.is_active },
    })
  }

  const handleToggleMaintenance = (app: AdminApplication) => {
    updateMutation.mutate({
      appId: app.id,
      data: { maintenance_mode: !app.maintenance_mode },
    })
  }

  const handleMoveUp = (index: number) => {
    if (!applications || index <= 0) return
    reorderMutation.mutate({
      appId: applications[index].id,
      targetAppId: applications[index - 1].id,
    })
  }

  const handleMoveDown = (index: number) => {
    if (!applications || index >= applications.length - 1) return
    reorderMutation.mutate({
      appId: applications[index].id,
      targetAppId: applications[index + 1].id,
    })
  }

  const openEditDialog = (app: AdminApplication) => {
    setEditForm({
      display_name: app.display_name,
      description: app.description ?? '',
      version: app.version ?? '',
      icon_url: app.icon_url ?? '',
      source_code_url: app.source_code_url ?? '',
      subdomain: app.subdomain ?? '',
      container_name: app.container_name,
      health_check_url: app.health_check_url ?? '',
      webhook_url: app.webhook_url ?? '',
      maintenance_message: app.maintenance_message ?? '',
      forgejo_owner: app.forgejo_owner ?? '',
      forgejo_repo: app.forgejo_repo ?? '',
      pinned_release_tag: app.pinned_release_tag ?? '',
    })
    setRefreshResult(null)
    setRefreshError('')
    setEditingApp(app)
  }

  const forgejoOwner = editForm.forgejo_owner ?? ''
  const forgejoRepo = editForm.forgejo_repo ?? ''
  const pinnedReleaseTag = editForm.pinned_release_tag ?? ''

  const forgejoValues = [forgejoOwner, forgejoRepo, pinnedReleaseTag]
  const forgejoAnyFilled = forgejoValues.some((v) => v !== '')
  const forgejoAllFilled = forgejoValues.every((v) => v !== '')
  const forgejoValidationError =
    forgejoAnyFilled && !forgejoAllFilled
      ? 'forgejo_owner, forgejo_repo, and pinned_release_tag must all be set together'
      : ''

  const handleEditSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!editingApp) return
    if (forgejoValidationError) return
    const data: UpdateApplicationRequest = {
      ...editForm,
      forgejo_owner: forgejoOwner || null,
      forgejo_repo: forgejoRepo || null,
      pinned_release_tag: pinnedReleaseTag || null,
    }
    editMutation.mutate({ appId: editingApp.id, data })
  }

  const handleRefreshRelease = async () => {
    if (!editingApp) return
    setIsRefreshing(true)
    setRefreshResult(null)
    setRefreshError('')
    try {
      const result = await downloadsApi.adminRefresh(editingApp.slug)
      setRefreshResult(result)
    } catch (err) {
      const apiErr = err as ApiError
      setRefreshError(apiErr?.error?.message || 'Failed to refresh release')
    } finally {
      setIsRefreshing(false)
    }
  }

  const handleCreateNameChange = (name: string) => {
    const slug = name.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '')
    setCreateForm({ ...createForm, name, slug, display_name: name })
  }

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    createMutation.mutate(createForm)
  }

  const openDeleteDialog = (app: AdminApplication) => {
    setDeletingApp(app)
    setDeleteStep(1)
    setDeletePassword('')
    setDeleteTotpCode('')
    setDeleteError('')
  }

  const closeDeleteDialog = () => {
    setDeletingApp(null)
    setDeleteStep(1)
    setDeletePassword('')
    setDeleteTotpCode('')
    setDeleteError('')
  }

  const handleDeletePasswordStep = (e: React.FormEvent) => {
    e.preventDefault()
    setDeleteError('')
    setDeleteStep(2)
  }

  const handleDeleteConfirm = (e: React.FormEvent) => {
    e.preventDefault()
    if (!deletingApp) return
    setDeleteError('')
    deleteMutation.mutate({
      appId: deletingApp.id,
      password: deletePassword,
      totp_code: deleteTotpCode,
    })
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Applications</h1>
          <p className="mt-2 text-muted-foreground">
            Manage platform applications.
          </p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Add Application
        </Button>
      </div>

      {isError && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Failed to load applications</AlertTitle>
          <AlertDescription>{(error as unknown as ApiError)?.error?.message || 'Could not connect to the API. Please try again later.'}</AlertDescription>
        </Alert>
      )}

      <div className="grid gap-6">
        {applications?.map((app, index) => (
          <Card key={app.id}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="flex flex-col gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6"
                      disabled={index === 0 || reorderMutation.isPending}
                      onClick={() => handleMoveUp(index)}
                    >
                      <ArrowUp className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6"
                      disabled={index === (applications?.length ?? 0) - 1 || reorderMutation.isPending}
                      onClick={() => handleMoveDown(index)}
                    >
                      <ArrowDown className="h-4 w-4" />
                    </Button>
                  </div>
                  <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                    <AppWindow className="h-6 w-6 text-primary" />
                  </div>
                  <div>
                    <CardTitle>{app.display_name}</CardTitle>
                    <CardDescription>{app.subdomain || app.slug}.{config.appDomain}</CardDescription>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  {app.maintenance_mode && (
                    <Badge variant="warning">Maintenance</Badge>
                  )}
                  {app.is_active ? (
                    <Badge variant="success">Active</Badge>
                  ) : (
                    <Badge variant="secondary">Inactive</Badge>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground mb-4">{app.description}</p>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-6">
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">Active</span>
                    <Switch
                      checked={app.is_active}
                      onCheckedChange={() => handleToggleActive(app)}
                      disabled={updateMutation.isPending}
                    />
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">
                      Maintenance Mode
                    </span>
                    <Switch
                      checked={app.maintenance_mode}
                      onCheckedChange={() => handleToggleMaintenance(app)}
                      disabled={updateMutation.isPending}
                    />
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="outline" size="sm" onClick={() => openEditDialog(app)}>
                    <Pencil className="mr-2 h-4 w-4" />
                    Edit
                  </Button>
                  <Button variant="destructive" size="sm" onClick={() => openDeleteDialog(app)}>
                    <Trash2 className="mr-2 h-4 w-4" />
                    Delete
                  </Button>
                  <a
                    href={`https://${app.subdomain || app.slug}.${config.appDomain}`}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <Button variant="outline" size="sm">
                      Open
                      <ExternalLink className="ml-2 h-4 w-4" />
                    </Button>
                  </a>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Create Application Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={(open) => {
        if (!open) {
          setShowCreateDialog(false)
          setCreateForm({ name: '', slug: '', display_name: '', container_name: '' })
          createMutation.reset()
        }
      }}>
        <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-lg">
          <form onSubmit={handleCreateSubmit}>
            <DialogHeader>
              <DialogTitle>Add Application</DialogTitle>
              <DialogDescription>
                Register a new application on the platform.
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="create_name">Name *</Label>
                <Input
                  id="create_name"
                  required
                  value={createForm.name}
                  onChange={(e) => handleCreateNameChange(e.target.value)}
                  placeholder="My Application"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_slug">Slug *</Label>
                <Input
                  id="create_slug"
                  required
                  value={createForm.slug}
                  onChange={(e) => setCreateForm({ ...createForm, slug: e.target.value })}
                  placeholder="my-application"
                  pattern="^[a-z0-9-]+$"
                  title="Lowercase letters, numbers, and hyphens only"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_display_name">Display Name *</Label>
                <Input
                  id="create_display_name"
                  required
                  value={createForm.display_name}
                  onChange={(e) => setCreateForm({ ...createForm, display_name: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_description">Description</Label>
                <Textarea
                  id="create_description"
                  value={createForm.description ?? ''}
                  onChange={(e) => setCreateForm({ ...createForm, description: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_container_name">Container Name *</Label>
                <Input
                  id="create_container_name"
                  required
                  value={createForm.container_name}
                  onChange={(e) => setCreateForm({ ...createForm, container_name: e.target.value })}
                  placeholder="my-app-container"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_subdomain">Subdomain</Label>
                <div className="flex items-center gap-2">
                  <Input
                    id="create_subdomain"
                    value={createForm.subdomain ?? ''}
                    onChange={(e) => setCreateForm({ ...createForm, subdomain: e.target.value })}
                    placeholder={createForm.slug || 'my-app'}
                  />
                  <span className="text-sm text-muted-foreground whitespace-nowrap">.{config.appDomain}</span>
                </div>
                <p className="text-xs text-muted-foreground">
                  Leave empty to use the slug as the subdomain.
                </p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_health_check_url">Health Check URL</Label>
                <Input
                  id="create_health_check_url"
                  value={createForm.health_check_url ?? ''}
                  onChange={(e) => setCreateForm({ ...createForm, health_check_url: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_webhook_url">Webhook URL</Label>
                <Input
                  id="create_webhook_url"
                  value={createForm.webhook_url ?? ''}
                  onChange={(e) => setCreateForm({ ...createForm, webhook_url: e.target.value })}
                  placeholder="https://app.example.com/webhooks/platform"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_version">Version</Label>
                <Input
                  id="create_version"
                  value={createForm.version ?? ''}
                  onChange={(e) => setCreateForm({ ...createForm, version: e.target.value })}
                  placeholder="1.0.0"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_source_code_url">Source Code URL</Label>
                <Input
                  id="create_source_code_url"
                  value={createForm.source_code_url ?? ''}
                  onChange={(e) => setCreateForm({ ...createForm, source_code_url: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="create_icon_url">Icon URL</Label>
                <Input
                  id="create_icon_url"
                  value={createForm.icon_url ?? ''}
                  onChange={(e) => setCreateForm({ ...createForm, icon_url: e.target.value })}
                />
              </div>
            </div>
            {createMutation.isError && (
              <Alert variant="destructive" className="mb-4">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {(createMutation.error as unknown as ApiError)?.error?.message || 'Failed to create application'}
                </AlertDescription>
              </Alert>
            )}
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setShowCreateDialog(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create Application
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Edit Application Dialog */}
      <Dialog open={!!editingApp} onOpenChange={(open) => !open && setEditingApp(null)}>
        <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-lg">
          <form onSubmit={handleEditSubmit}>
            <DialogHeader>
              <DialogTitle>Edit {editingApp?.display_name}</DialogTitle>
              <DialogDescription>
                Update application settings for {editingApp?.slug}.
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="display_name">Display Name *</Label>
                <Input
                  id="display_name"
                  required
                  value={editForm.display_name ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, display_name: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="subdomain">Subdomain</Label>
                <div className="flex items-center gap-2">
                  <Input
                    id="subdomain"
                    placeholder={editingApp?.slug}
                    value={editForm.subdomain ?? ''}
                    onChange={(e) => setEditForm({ ...editForm, subdomain: e.target.value })}
                  />
                  <span className="text-sm text-muted-foreground whitespace-nowrap">.{config.appDomain}</span>
                </div>
                <p className="text-xs text-muted-foreground">
                  Leave empty to use the slug ({editingApp?.slug}) as the subdomain.
                </p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  value={editForm.description ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, description: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="version">Version</Label>
                <Input
                  id="version"
                  value={editForm.version ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, version: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="icon_url">Icon URL</Label>
                <Input
                  id="icon_url"
                  value={editForm.icon_url ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, icon_url: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="source_code_url">Source Code URL</Label>
                <Input
                  id="source_code_url"
                  value={editForm.source_code_url ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, source_code_url: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="container_name">Container Name *</Label>
                <Input
                  id="container_name"
                  required
                  value={editForm.container_name ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, container_name: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="health_check_url">Health Check URL</Label>
                <Input
                  id="health_check_url"
                  value={editForm.health_check_url ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, health_check_url: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="webhook_url">Webhook URL</Label>
                <Input
                  id="webhook_url"
                  placeholder="https://app.example.com/webhooks/platform"
                  value={editForm.webhook_url ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, webhook_url: e.target.value })}
                />
                <p className="text-xs text-muted-foreground">
                  Platform will POST to this URL when maintenance mode or active status changes.
                </p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="forgejo_owner">Forgejo Owner</Label>
                <Input
                  id="forgejo_owner"
                  value={editForm.forgejo_owner ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, forgejo_owner: e.target.value })}
                  placeholder="owner"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="forgejo_repo">Forgejo Repo</Label>
                <Input
                  id="forgejo_repo"
                  value={editForm.forgejo_repo ?? ''}
                  onChange={(e) => setEditForm({ ...editForm, forgejo_repo: e.target.value })}
                  placeholder="repo"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="pinned_release_tag">Pinned Release Tag</Label>
                <div className="flex items-center gap-2">
                  <Input
                    id="pinned_release_tag"
                    value={editForm.pinned_release_tag ?? ''}
                    onChange={(e) => setEditForm({ ...editForm, pinned_release_tag: e.target.value })}
                    placeholder="v1.0.0 or latest"
                  />
                  {forgejoAllFilled && editingApp && (
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handleRefreshRelease}
                      disabled={isRefreshing}
                    >
                      {isRefreshing ? (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      ) : (
                        <RefreshCw className="mr-2 h-4 w-4" />
                      )}
                      Refresh release
                    </Button>
                  )}
                </div>
              </div>
              {forgejoValidationError && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{forgejoValidationError}</AlertDescription>
                </Alert>
              )}
              {refreshError && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{refreshError}</AlertDescription>
                </Alert>
              )}
              {refreshResult && (
                <div className="rounded-md border p-3 text-sm">
                  <p className="font-medium mb-2">Release: {refreshResult.release_tag ?? 'unknown'}</p>
                  {refreshResult.assets.length === 0 ? (
                    <p className="text-muted-foreground">No assets found.</p>
                  ) : (
                    <ul className="space-y-1">
                      {refreshResult.assets.map((asset) => (
                        <li key={asset.asset_name} className="flex items-center justify-between gap-4">
                          <span className="font-mono text-xs">{asset.asset_name}</span>
                          <span className="text-muted-foreground text-xs whitespace-nowrap">
                            {(asset.size_bytes / 1024).toFixed(0)} KB
                          </span>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
              {editingApp?.maintenance_mode && (
                <div className="grid gap-2">
                  <Label htmlFor="maintenance_message">Maintenance Message</Label>
                  <Textarea
                    id="maintenance_message"
                    value={editForm.maintenance_message ?? ''}
                    onChange={(e) => setEditForm({ ...editForm, maintenance_message: e.target.value })}
                  />
                </div>
              )}
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setEditingApp(null)}>
                Cancel
              </Button>
              <Button type="submit" disabled={editMutation.isPending || !!forgejoValidationError}>
                {editMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Save Changes
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete Application Dialog */}
      <Dialog open={!!deletingApp} onOpenChange={(open) => !open && closeDeleteDialog()}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-destructive">
              <ShieldAlert className="h-5 w-5" />
              Delete {deletingApp?.display_name}
            </DialogTitle>
            <DialogDescription>
              This action cannot be undone. The application and all associated data will be permanently removed.
            </DialogDescription>
          </DialogHeader>

          {!twoFactorStatus?.enabled ? (
            <div className="py-4">
              <Alert>
                <ShieldAlert className="h-4 w-4" />
                <AlertTitle>2FA Required</AlertTitle>
                <AlertDescription>
                  You must enable two-factor authentication before you can delete applications. Go to your security settings to enable 2FA.
                </AlertDescription>
              </Alert>
              <DialogFooter className="mt-4">
                <Button variant="outline" onClick={closeDeleteDialog}>
                  Close
                </Button>
              </DialogFooter>
            </div>
          ) : (
            <form onSubmit={deleteStep === 1 ? handleDeletePasswordStep : handleDeleteConfirm} autoComplete="off">
              <div className="grid gap-4 py-4">
                <div className={deleteStep === 1 ? undefined : 'hidden'}>
                  <Alert variant="destructive" className="mb-4">
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                      You are about to delete <strong>{deletingApp?.display_name}</strong> ({deletingApp?.slug}). This action is irreversible.
                    </AlertDescription>
                  </Alert>
                  <div className="grid gap-2">
                    <Label htmlFor="delete_password">Confirm your password</Label>
                    <Input
                      id="delete_password"
                      type="password"
                      required={deleteStep === 1}
                      value={deletePassword}
                      onChange={(e) => setDeletePassword(e.target.value)}
                      autoComplete="off"
                      autoFocus
                    />
                  </div>
                </div>
                {deleteStep === 2 && (
                  <div className="grid gap-2">
                    <Label htmlFor="delete_totp">Enter your 2FA code</Label>
                    <Input
                      id="delete_totp"
                      required
                      value={deleteTotpCode}
                      onChange={(e) => {
                        const val = e.target.value.replace(/\D/g, '').slice(0, 6)
                        setDeleteTotpCode(val)
                        if (val.length === 6 && deletingApp && !deleteMutation.isPending) {
                          setDeleteError('')
                          deleteMutation.mutate({
                            appId: deletingApp.id,
                            password: deletePassword,
                            totp_code: val,
                          })
                        }
                      }}
                      placeholder="000000"
                      maxLength={6}
                      pattern="[0-9]{6}"
                      title="Enter a 6-digit code"
                      autoComplete="off"
                      autoFocus
                    />
                  </div>
                )}
              </div>
              {deleteError && (
                <Alert variant="destructive" className="mb-4">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{deleteError}</AlertDescription>
                </Alert>
              )}
              <DialogFooter>
                {deleteStep === 1 ? (
                  <>
                    <Button type="button" variant="outline" onClick={closeDeleteDialog}>
                      Cancel
                    </Button>
                    <Button type="submit" variant="destructive">
                      Next
                    </Button>
                  </>
                ) : (
                  <>
                    <Button type="button" variant="outline" onClick={() => { setDeleteStep(1); setDeleteError('') }}>
                      Back
                    </Button>
                    <Button type="submit" variant="destructive" disabled={deleteMutation.isPending}>
                      {deleteMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                      Delete Application
                    </Button>
                  </>
                )}
              </DialogFooter>
            </form>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
