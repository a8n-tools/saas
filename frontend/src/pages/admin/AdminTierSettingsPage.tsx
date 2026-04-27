import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Settings, CheckCircle, AlertCircle } from 'lucide-react'
import { adminApi } from '@/api/admin'
import type { UpdateTierConfigRequest } from '@/api/admin'
import type { ApiError } from '@/types'

export default function AdminTierSettingsPage() {
  const queryClient = useQueryClient()
  const [form, setForm] = useState<UpdateTierConfigRequest>({})
  const [saveSuccess, setSaveSuccess] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)

  const { data: config, isLoading } = useQuery({
    queryKey: ['admin', 'tier-config'],
    queryFn: adminApi.getTierConfig,
  })

  const updateMutation = useMutation({
    mutationFn: (data: UpdateTierConfigRequest) => adminApi.updateTierConfig(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'tier-config'] })
      setForm({})
      setSaveSuccess(true)
      setSaveError(null)
      setTimeout(() => setSaveSuccess(false), 4000)
    },
    onError: (err) => {
      const apiError = err as unknown as ApiError
      setSaveError(apiError?.error?.message ?? 'Failed to save tier configuration.')
      setSaveSuccess(false)
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setSaveSuccess(false)
    setSaveError(null)

    const payload: UpdateTierConfigRequest = {}
    if (form.lifetime_slots !== undefined) payload.lifetime_slots = form.lifetime_slots
    if (form.early_adopter_slots !== undefined) payload.early_adopter_slots = form.early_adopter_slots
    if (form.early_adopter_trial_days !== undefined) payload.early_adopter_trial_days = form.early_adopter_trial_days
    if (form.standard_trial_days !== undefined) payload.standard_trial_days = form.standard_trial_days

    if (Object.keys(payload).length === 0) return
    updateMutation.mutate(payload)
  }

  const setField = (field: keyof UpdateTierConfigRequest, value: string) => {
    const num = value === '' ? undefined : parseInt(value, 10)
    if (num !== undefined && isNaN(num)) return
    setForm((prev) => ({ ...prev, [field]: num }))
  }

  if (isLoading) {
    return (
      <div className="space-y-6">
        <h1 className="text-2xl font-bold">Tier Settings</h1>
        <p className="text-muted-foreground">Loading...</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Tier Settings</h1>
          <p className="text-muted-foreground">
            Configure membership tiers assigned to new users upon email verification.
          </p>
        </div>
        {config && (
          <Badge variant={config.source === 'database' ? 'default' : 'secondary'}>
            Source: {config.source}
          </Badge>
        )}
      </div>

      {config && (
        <div className="grid gap-4 sm:grid-cols-2">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Lifetime Slots
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">
                {config.lifetime_slots_used}
                <span className="text-muted-foreground font-normal"> / {config.lifetime_slots}</span>
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                {config.lifetime_slots - config.lifetime_slots_used > 0
                  ? `${config.lifetime_slots - config.lifetime_slots_used} remaining`
                  : 'All slots filled'}
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Early Adopter Slots
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-2xl font-bold">
                {config.early_adopter_slots_used}
                <span className="text-muted-foreground font-normal"> / {config.early_adopter_slots}</span>
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                {config.early_adopter_slots - config.early_adopter_slots_used > 0
                  ? `${config.early_adopter_slots - config.early_adopter_slots_used} remaining`
                  : 'All slots filled'}
              </p>
            </CardContent>
          </Card>
        </div>
      )}

      <form onSubmit={handleSubmit}>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Settings className="h-4 w-4" />
              Tier Thresholds
            </CardTitle>
            <CardDescription>
              The first users to verify their email are assigned tiers in order: lifetime
              (free forever), then early adopter (extended trial), then standard. Adjust
              the slot counts and trial durations below. Changes take effect immediately
              for the next user verification.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-5">
            <div className="grid gap-5 sm:grid-cols-2">
              <div className="space-y-1.5">
                <Label htmlFor="lifetime_slots">Lifetime Slots</Label>
                <Input
                  id="lifetime_slots"
                  type="number"
                  min={0}
                  placeholder={String(config?.lifetime_slots ?? 5)}
                  value={form.lifetime_slots ?? ''}
                  onChange={(e) => setField('lifetime_slots', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Number of users who get free lifetime membership
                </p>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="early_adopter_slots">Early Adopter Slots</Label>
                <Input
                  id="early_adopter_slots"
                  type="number"
                  min={0}
                  placeholder={String(config?.early_adopter_slots ?? 5)}
                  value={form.early_adopter_slots ?? ''}
                  onChange={(e) => setField('early_adopter_slots', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Number of users who get the early adopter trial
                </p>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="early_adopter_trial_days">Early Adopter Trial (days)</Label>
                <Input
                  id="early_adopter_trial_days"
                  type="number"
                  min={0}
                  placeholder={String(config?.early_adopter_trial_days ?? 90)}
                  value={form.early_adopter_trial_days ?? ''}
                  onChange={(e) => setField('early_adopter_trial_days', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Trial duration for early adopter tier
                </p>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="standard_trial_days">Standard Trial (days)</Label>
                <Input
                  id="standard_trial_days"
                  type="number"
                  min={0}
                  placeholder={String(config?.standard_trial_days ?? 30)}
                  value={form.standard_trial_days ?? ''}
                  onChange={(e) => setField('standard_trial_days', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Trial duration for standard tier
                </p>
              </div>
            </div>

            {saveSuccess && (
              <Alert>
                <CheckCircle className="h-4 w-4" />
                <AlertDescription>Tier configuration saved and applied.</AlertDescription>
              </Alert>
            )}

            {saveError && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{saveError}</AlertDescription>
              </Alert>
            )}

            <div className="flex justify-end">
              <Button type="submit" disabled={updateMutation.isPending}>
                {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
              </Button>
            </div>
          </CardContent>
        </Card>
      </form>
    </div>
  )
}
