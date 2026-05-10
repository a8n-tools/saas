import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { CheckCircle, AlertCircle, Crown, Users, Star } from 'lucide-react'
import { adminApi } from '@/api/admin'
import type { UpdateTierConfigRequest } from '@/api/admin'
import type { ApiError } from '@/types'

const UNSET = '__unset__'

export default function AdminTierSettingsPage() {
  const queryClient = useQueryClient()
  const [form, setForm] = useState<UpdateTierConfigRequest>({})
  const [saveSuccess, setSaveSuccess] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)

  const { data: config, isLoading } = useQuery({
    queryKey: ['admin', 'tier-config'],
    queryFn: adminApi.getTierConfig,
  })

  const { data: products = [] } = useQuery({
    queryKey: ['admin', 'stripe-products'],
    queryFn: adminApi.listStripeProducts,
    retry: false,
  })

  const activeProducts = products.filter((p) => p.active)

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
    if (form.lifetime_product_id !== undefined) payload.lifetime_product_id = form.lifetime_product_id
    if (form.early_adopter_product_id !== undefined) payload.early_adopter_product_id = form.early_adopter_product_id
    if (form.standard_product_id !== undefined) payload.standard_product_id = form.standard_product_id

    if (Object.keys(payload).length === 0) return
    updateMutation.mutate(payload)
  }

  const setNumField = (field: keyof UpdateTierConfigRequest, value: string) => {
    const num = value === '' ? undefined : parseInt(value, 10)
    if (num !== undefined && isNaN(num)) return
    setForm((prev) => ({ ...prev, [field]: num }))
  }

  const setProductField = (
    field: 'lifetime_product_id' | 'early_adopter_product_id' | 'standard_product_id',
    value: string,
  ) => {
    setForm((prev) => ({ ...prev, [field]: value === UNSET ? undefined : value }))
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

      <form onSubmit={handleSubmit} className="space-y-4">
        {/* Lifetime tier card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Crown className="h-4 w-4" />
              Lifetime
            </CardTitle>
            <CardDescription>
              The first users to verify their email receive a free lifetime membership. Associate a
              Stripe product so that subscription webhooks for that product automatically set the
              Lifetime tier.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {config && (
              <p className="text-sm text-muted-foreground">
                {config.lifetime_slots_used} / {config.lifetime_slots} slots used
                {config.lifetime_slots - config.lifetime_slots_used > 0
                  ? ` (${config.lifetime_slots - config.lifetime_slots_used} remaining)`
                  : ' — all filled'}
              </p>
            )}
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-1.5">
                <Label>Stripe Product</Label>
                <Select
                  value={form.lifetime_product_id ?? config?.lifetime_product_id ?? UNSET}
                  onValueChange={(v) => setProductField('lifetime_product_id', v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Not configured" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={UNSET}>Not configured</SelectItem>
                    {activeProducts.map((p) => (
                      <SelectItem key={p.id} value={p.id}>
                        {p.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  Subscription webhooks for this product set tier to Lifetime
                </p>
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="lifetime_slots">Lifetime Slots</Label>
                <Input
                  id="lifetime_slots"
                  type="number"
                  min={0}
                  placeholder={String(config?.lifetime_slots ?? 5)}
                  value={form.lifetime_slots ?? ''}
                  onChange={(e) => setNumField('lifetime_slots', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Number of users who get free lifetime membership
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Early Adopter tier card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Star className="h-4 w-4" />
              Early Adopter
            </CardTitle>
            <CardDescription>
              The next users after the lifetime slots receive an extended trial. Associate a Stripe
              product so subscription webhooks automatically set the Early Adopter tier.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {config && (
              <p className="text-sm text-muted-foreground">
                {config.early_adopter_slots_used} / {config.early_adopter_slots} slots used
                {config.early_adopter_slots - config.early_adopter_slots_used > 0
                  ? ` (${config.early_adopter_slots - config.early_adopter_slots_used} remaining)`
                  : ' — all filled'}
              </p>
            )}
            <div className="grid gap-4 sm:grid-cols-3">
              <div className="space-y-1.5">
                <Label>Stripe Product</Label>
                <Select
                  value={form.early_adopter_product_id ?? config?.early_adopter_product_id ?? UNSET}
                  onValueChange={(v) => setProductField('early_adopter_product_id', v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Not configured" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={UNSET}>Not configured</SelectItem>
                    {activeProducts.map((p) => (
                      <SelectItem key={p.id} value={p.id}>
                        {p.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  Subscription webhooks for this product set tier to Early Adopter
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
                  onChange={(e) => setNumField('early_adopter_slots', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Number of users who get the early adopter trial
                </p>
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="early_adopter_trial_days">Trial (days)</Label>
                <Input
                  id="early_adopter_trial_days"
                  type="number"
                  min={0}
                  placeholder={String(config?.early_adopter_trial_days ?? 90)}
                  value={form.early_adopter_trial_days ?? ''}
                  onChange={(e) => setNumField('early_adopter_trial_days', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Trial duration for early adopter tier
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Standard tier card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Users className="h-4 w-4" />
              Standard
            </CardTitle>
            <CardDescription>
              All remaining users receive a standard trial. Associate a Stripe product so
              subscription webhooks automatically set the Standard tier.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-1.5">
                <Label>Stripe Product</Label>
                <Select
                  value={form.standard_product_id ?? config?.standard_product_id ?? UNSET}
                  onValueChange={(v) => setProductField('standard_product_id', v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Not configured" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={UNSET}>Not configured</SelectItem>
                    {activeProducts.map((p) => (
                      <SelectItem key={p.id} value={p.id}>
                        {p.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  Subscription webhooks for this product set tier to Standard
                </p>
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="standard_trial_days">Trial (days)</Label>
                <Input
                  id="standard_trial_days"
                  type="number"
                  min={0}
                  placeholder={String(config?.standard_trial_days ?? 30)}
                  value={form.standard_trial_days ?? ''}
                  onChange={(e) => setNumField('standard_trial_days', e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Trial duration for standard tier
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

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
      </form>
    </div>
  )
}
