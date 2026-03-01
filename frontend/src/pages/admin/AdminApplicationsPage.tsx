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
import { Loader2, AppWindow, ExternalLink, Pencil, AlertCircle } from 'lucide-react'
import { adminApi } from '@/api/admin'
import type { AdminApplication, UpdateApplicationRequest } from '@/api/admin'
import { config } from '@/config'
import type { ApiError } from '@/types'

export function AdminApplicationsPage() {
  const queryClient = useQueryClient()
  const [editingApp, setEditingApp] = useState<AdminApplication | null>(null)
  const [editForm, setEditForm] = useState<UpdateApplicationRequest>({})

  const { data: applications, isLoading, isError, error } = useQuery({
    queryKey: ['admin', 'applications'],
    queryFn: adminApi.getApplications,
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

  const openEditDialog = (app: AdminApplication) => {
    setEditForm({
      display_name: app.display_name,
      description: app.description ?? '',
      version: app.version ?? '',
      icon_url: app.icon_url ?? '',
      source_code_url: app.source_code_url ?? '',
      container_name: app.container_name,
      health_check_url: app.health_check_url ?? '',
      maintenance_message: app.maintenance_message ?? '',
    })
    setEditingApp(app)
  }

  const handleEditSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!editingApp) return
    editMutation.mutate({ appId: editingApp.id, data: editForm })
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
      <div>
        <h1 className="text-3xl font-bold">Applications</h1>
        <p className="mt-2 text-muted-foreground">
          Manage platform applications.
        </p>
      </div>

      {isError && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Failed to load applications</AlertTitle>
          <AlertDescription>{(error as unknown as ApiError)?.error?.message || 'Could not connect to the API. Please try again later.'}</AlertDescription>
        </Alert>
      )}

      <div className="grid gap-6">
        {applications?.map((app) => (
          <Card key={app.id}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                    <AppWindow className="h-6 w-6 text-primary" />
                  </div>
                  <div>
                    <CardTitle>{app.display_name}</CardTitle>
                    <CardDescription>{app.slug}</CardDescription>
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
                  <a
                    href={`https://${app.slug}.${config.appDomain}`}
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
              <Button type="submit" disabled={editMutation.isPending}>
                {editMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Save Changes
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}
