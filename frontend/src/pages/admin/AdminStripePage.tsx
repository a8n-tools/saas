import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Loader2, AlertCircle, CheckCircle2, Banknote } from 'lucide-react'
import { adminApi } from '@/api/admin'
import type { UpdateStripeConfigRequest } from '@/api/admin'
import type { ApiError } from '@/types'

export function AdminStripePage() {
  const queryClient = useQueryClient()

  const [form, setForm] = useState<UpdateStripeConfigRequest>({
    secret_key: '',
    webhook_secret: '',
    price_id_personal: '',
    price_id_business: '',
  })
  const [saveSuccess, setSaveSuccess] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)

  const { data: config, isLoading, isError, error } = useQuery({
    queryKey: ['admin', 'stripe'],
    queryFn: adminApi.getStripeConfig,
  })

  const updateMutation = useMutation({
    mutationFn: (data: UpdateStripeConfigRequest) => adminApi.updateStripeConfig(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe'] })
      setForm({ secret_key: '', webhook_secret: '', price_id_personal: '', price_id_business: '' })
      setSaveSuccess(true)
      setSaveError(null)
      setTimeout(() => setSaveSuccess(false), 4000)
    },
    onError: (err) => {
      const apiError = err as unknown as ApiError
      setSaveError(apiError?.error?.message ?? 'Failed to save Stripe configuration.')
      setSaveSuccess(false)
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setSaveSuccess(false)
    setSaveError(null)
    // Only send fields the user actually typed into
    const payload: UpdateStripeConfigRequest = {}
    if (form.secret_key) payload.secret_key = form.secret_key
    if (form.webhook_secret) payload.webhook_secret = form.webhook_secret
    if (form.price_id_personal) payload.price_id_personal = form.price_id_personal
    if (form.price_id_business) payload.price_id_business = form.price_id_business
    updateMutation.mutate(payload)
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (isError) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>Failed to load Stripe configuration</AlertTitle>
        <AlertDescription>
          {(error as unknown as ApiError)?.error?.message ?? 'Could not connect to the API.'}
        </AlertDescription>
      </Alert>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Stripe</h1>
        <p className="text-muted-foreground">
          Manage your Stripe payment integration settings. Changes take effect immediately without a server restart.
        </p>
      </div>

      {config && (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <span>Current source:</span>
          <Badge variant={config.source === 'database' ? 'default' : 'secondary'}>
            {config.source === 'database' ? 'Database' : 'Environment variables'}
          </Badge>
          {config.updated_at && (
            <span className="text-xs">
              · Last updated {new Date(config.updated_at).toLocaleString()}
            </span>
          )}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Banknote className="h-4 w-4" />
              API Keys
            </CardTitle>
            <CardDescription>
              Leave a field blank to keep the existing value. Only fields you type into will be updated.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-5">
            <div className="space-y-1.5">
              <Label htmlFor="secret_key">Secret Key</Label>
              <Input
                id="secret_key"
                type="password"
                placeholder={
                  config?.has_secret_key
                    ? `Current: ${config.secret_key_masked} — leave blank to keep`
                    : 'sk_live_...'
                }
                value={form.secret_key}
                onChange={(e) => setForm({ ...form, secret_key: e.target.value })}
                autoComplete="off"
              />
              {config?.has_secret_key && (
                <p className="text-xs text-muted-foreground">
                  Currently set: <code className="font-mono">{config.secret_key_masked}</code>
                </p>
              )}
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="webhook_secret">Webhook Secret</Label>
              <Input
                id="webhook_secret"
                type="password"
                placeholder={
                  config?.has_webhook_secret
                    ? `Current: ${config.webhook_secret_masked} — leave blank to keep`
                    : 'whsec_...'
                }
                value={form.webhook_secret}
                onChange={(e) => setForm({ ...form, webhook_secret: e.target.value })}
                autoComplete="off"
              />
              {config?.has_webhook_secret && (
                <p className="text-xs text-muted-foreground">
                  Currently set: <code className="font-mono">{config.webhook_secret_masked}</code>
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Price IDs</CardTitle>
            <CardDescription>
              Stripe Price IDs for each subscription tier.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-5">
            <div className="space-y-1.5">
              <Label htmlFor="price_id_personal">Personal Plan Price ID</Label>
              <Input
                id="price_id_personal"
                placeholder={config?.price_id_personal ?? 'price_...'}
                value={form.price_id_personal}
                onChange={(e) => setForm({ ...form, price_id_personal: e.target.value })}
              />
              {config?.price_id_personal && (
                <p className="text-xs text-muted-foreground">
                  Currently set: <code className="font-mono">{config.price_id_personal}</code>
                </p>
              )}
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="price_id_business">Business Plan Price ID</Label>
              <Input
                id="price_id_business"
                placeholder={config?.price_id_business ?? 'price_...'}
                value={form.price_id_business}
                onChange={(e) => setForm({ ...form, price_id_business: e.target.value })}
              />
              {config?.price_id_business && (
                <p className="text-xs text-muted-foreground">
                  Currently set: <code className="font-mono">{config.price_id_business}</code>
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        {saveSuccess && (
          <Alert>
            <CheckCircle2 className="h-4 w-4" />
            <AlertTitle>Saved</AlertTitle>
            <AlertDescription>Stripe configuration updated successfully.</AlertDescription>
          </Alert>
        )}

        {saveError && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Save failed</AlertTitle>
            <AlertDescription>{saveError}</AlertDescription>
          </Alert>
        )}

        <div className="flex justify-end">
          <Button type="submit" disabled={updateMutation.isPending}>
            {updateMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Save Changes
          </Button>
        </div>
      </form>
    </div>
  )
}
