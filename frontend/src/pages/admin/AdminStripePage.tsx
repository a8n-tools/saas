import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Loader2,
  AlertCircle,
  CheckCircle2,
  Banknote,
  Plus,
  MoreHorizontal,
  Pencil,
  Archive,
  Trash2,
  Key,
  Copy,
} from 'lucide-react'
import { adminApi } from '@/api/admin'
import type { UpdateStripeConfigRequest } from '@/api/admin'
import type { ApiError, StripeProduct } from '@/types'

// ── Helpers ──────────────────────────────────────────────────────────────

function formatCurrency(amountCents: number, currency: string): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency.toUpperCase(),
  }).format(amountCents / 100)
}

function formatDate(epoch: number): string {
  return new Date(epoch * 1000).toLocaleDateString()
}

const WEBHOOK_EVENTS = [
  'checkout.session.completed',
  'customer.subscription.created',
  'customer.subscription.updated',
  'customer.subscription.deleted',
  'invoice.payment_succeeded',
  'invoice.payment_failed',
] as const

// ── API Keys Tab ─────────────────────────────────────────────────────────

function ApiKeysTab() {
  const queryClient = useQueryClient()
  const [form, setForm] = useState<UpdateStripeConfigRequest>({ secret_key: '' })
  const [saveSuccess, setSaveSuccess] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)

  const { data: config } = useQuery({
    queryKey: ['admin', 'stripe'],
    queryFn: adminApi.getStripeConfig,
  })

  const updateMutation = useMutation({
    mutationFn: (data: UpdateStripeConfigRequest) => adminApi.updateStripeConfig(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe'] })
      setForm({ secret_key: '' })
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
    const payload: UpdateStripeConfigRequest = {}
    if (form.secret_key) payload.secret_key = form.secret_key
    updateMutation.mutate(payload)
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <Banknote className="h-4 w-4" />
            API Keys
          </CardTitle>
          <CardDescription>
            Your Stripe secret key authenticates API requests to Stripe. Generate a{' '}
            <a
              href="https://dashboard.stripe.com/apikeys"
              target="_blank"
              rel="noopener noreferrer"
              className="underline hover:text-foreground"
            >
              restricted key
            </a>
            {' '}with the following permissions: <strong>Products</strong>,{' '}
            <strong>Prices</strong>, <strong>Customers</strong>,{' '}
            <strong>Subscriptions</strong>, <strong>Checkout Sessions</strong>,{' '}
            and <strong>Webhook Endpoints</strong> set to <em>Write</em>;{' '}
            <strong>Invoices</strong> set to <em>Read</em>.
            Keys follow the format <code className="font-mono text-xs">rk_(live|test)_...</code> &mdash;
            the prefix indicates whether this is a live or test key.
            Leave the field blank to keep the existing value.
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
            <Label>Webhook Secret</Label>
            <Input
              type="text"
              readOnly
              value={config?.has_webhook_secret ? (config.webhook_secret_masked ?? '') : 'Not configured'}
              className="bg-muted"
            />
            <p className="text-xs text-muted-foreground">
              The signing secret used to verify incoming Stripe webhooks. Auto-populated when you create a webhook endpoint in the Webhooks tab.
            </p>
          </div>

          {config && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <span>Config source:</span>
              <Badge variant={config.source === 'database' ? 'default' : 'secondary'}>
                {config.source === 'database' ? 'Database' : 'Environment variables'}
              </Badge>
              {config.updated_at && (
                <span className="text-xs">
                  Last updated {new Date(config.updated_at).toLocaleString()}
                </span>
              )}
            </div>
          )}
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
  )
}

// ── Products Tab ─────────────────────────────────────────────────────────

function ProductsTab() {
  const queryClient = useQueryClient()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [editingProduct, setEditingProduct] = useState<StripeProduct | null>(null)
  const [createForm, setCreateForm] = useState({ name: '', description: '' })
  const [editForm, setEditForm] = useState({ name: '', description: '', active: true })
  const [archivingId, setArchivingId] = useState<string | null>(null)

  const { data: products, isLoading, isError } = useQuery({
    queryKey: ['admin', 'stripe', 'products'],
    queryFn: adminApi.listStripeProducts,
  })

  const createMutation = useMutation({
    mutationFn: adminApi.createStripeProduct,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe', 'products'] })
      setShowCreateDialog(false)
      setCreateForm({ name: '', description: '' })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Parameters<typeof adminApi.updateStripeProduct>[1] }) =>
      adminApi.updateStripeProduct(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe', 'products'] })
      setEditingProduct(null)
    },
  })

  const archiveMutation = useMutation({
    mutationFn: adminApi.archiveStripeProduct,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe', 'products'] })
      setArchivingId(null)
    },
  })

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    createMutation.mutate({
      name: createForm.name,
      description: createForm.description || undefined,
      metadata: {},
    })
  }

  const handleEditSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!editingProduct) return
    updateMutation.mutate({
      id: editingProduct.id,
      data: {
        name: editForm.name,
        description: editForm.description || undefined,
        active: editForm.active,
      },
    })
  }

  const openEditDialog = (product: StripeProduct) => {
    setEditForm({
      name: product.name,
      description: product.description ?? '',
      active: product.active,
    })
    setEditingProduct(product)
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
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          Failed to load products from Stripe. Check that your API key is valid and that all products have standard Stripe IDs.
        </AlertDescription>
      </Alert>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Manage Stripe products for subscription tiers.
        </p>
        <Button size="sm" onClick={() => setShowCreateDialog(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Product
        </Button>
      </div>

      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50">
                  <th className="px-4 py-3 text-left font-medium">Name</th>
                  <th className="px-4 py-3 text-left font-medium">Status</th>
                  <th className="px-4 py-3 text-left font-medium">Created</th>
                  <th className="px-4 py-3 text-right font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {products?.length === 0 && (
                  <tr>
                    <td colSpan={4} className="px-4 py-8 text-center text-muted-foreground">
                      No products found. Create one to get started.
                    </td>
                  </tr>
                )}
                {products?.map((product) => (
                  <tr key={product.id} className="border-b last:border-0">
                    <td className="px-4 py-3">
                      <div>
                        <p className="font-medium">{product.name}</p>
                        {product.description && (
                          <p className="text-xs text-muted-foreground">{product.description}</p>
                        )}
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <Badge variant={product.active ? 'default' : 'secondary'}>
                        {product.active ? 'Active' : 'Archived'}
                      </Badge>
                    </td>
                    <td className="px-4 py-3 text-muted-foreground">
                      {formatDate(product.created)}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon" className="h-8 w-8">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => openEditDialog(product)}>
                            <Pencil className="mr-2 h-4 w-4" />
                            Edit
                          </DropdownMenuItem>
                          {product.active && (
                            <DropdownMenuItem
                              className="text-destructive"
                              onClick={() => setArchivingId(product.id)}
                            >
                              <Archive className="mr-2 h-4 w-4" />
                              Archive
                            </DropdownMenuItem>
                          )}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Create Product Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={(open) => {
        if (!open) {
          setShowCreateDialog(false)
          setCreateForm({ name: '', description: '' })
          createMutation.reset()
        }
      }}>
        <DialogContent className="sm:max-w-md">
          <form onSubmit={handleCreateSubmit}>
            <DialogHeader>
              <DialogTitle>Create Product</DialogTitle>
              <DialogDescription>Create a new Stripe product for a subscription tier.</DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="product_name">Name</Label>
                <Input
                  id="product_name"
                  required
                  value={createForm.name}
                  onChange={(e) => setCreateForm({ ...createForm, name: e.target.value })}
                  placeholder="Personal Plan"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="product_description">Description</Label>
                <Textarea
                  id="product_description"
                  value={createForm.description}
                  onChange={(e) => setCreateForm({ ...createForm, description: e.target.value })}
                  placeholder="Optional description"
                />
              </div>
            </div>
            {createMutation.isError && (
              <Alert variant="destructive" className="mb-4">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {(createMutation.error as unknown as ApiError)?.error?.message || 'Failed to create product'}
                </AlertDescription>
              </Alert>
            )}
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setShowCreateDialog(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Edit Product Dialog */}
      <Dialog open={!!editingProduct} onOpenChange={(open) => !open && setEditingProduct(null)}>
        <DialogContent className="sm:max-w-md">
          <form onSubmit={handleEditSubmit}>
            <DialogHeader>
              <DialogTitle>Edit Product</DialogTitle>
              <DialogDescription>Update product details.</DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="edit_product_name">Name</Label>
                <Input
                  id="edit_product_name"
                  required
                  value={editForm.name}
                  onChange={(e) => setEditForm({ ...editForm, name: e.target.value })}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="edit_product_description">Description</Label>
                <Textarea
                  id="edit_product_description"
                  value={editForm.description}
                  onChange={(e) => setEditForm({ ...editForm, description: e.target.value })}
                />
              </div>
            </div>
            {updateMutation.isError && (
              <Alert variant="destructive" className="mb-4">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {(updateMutation.error as unknown as ApiError)?.error?.message || 'Failed to update product'}
                </AlertDescription>
              </Alert>
            )}
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setEditingProduct(null)}>
                Cancel
              </Button>
              <Button type="submit" disabled={updateMutation.isPending}>
                {updateMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Save Changes
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Archive Confirm Dialog */}
      <Dialog open={!!archivingId} onOpenChange={(open) => !open && setArchivingId(null)}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Archive Product</DialogTitle>
            <DialogDescription>
              Are you sure you want to archive this product? It will no longer be available for new subscriptions.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setArchivingId(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={archiveMutation.isPending}
              onClick={() => archivingId && archiveMutation.mutate(archivingId)}
            >
              {archiveMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Archive
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

// ── Prices Tab ───────────────────────────────────────────────────────────

function PricesTab() {
  const queryClient = useQueryClient()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [createForm, setCreateForm] = useState({
    product_id: '',
    amount: '',
    currency: 'usd',
    interval: 'month',
  })
  const [archivingId, setArchivingId] = useState<string | null>(null)

  const { data: prices, isLoading, isError } = useQuery({
    queryKey: ['admin', 'stripe', 'prices'],
    queryFn: () => adminApi.listStripePrices(),
  })

  const { data: products } = useQuery({
    queryKey: ['admin', 'stripe', 'products'],
    queryFn: adminApi.listStripeProducts,
  })

  const createMutation = useMutation({
    mutationFn: adminApi.createStripePrice,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe', 'prices'] })
      setShowCreateDialog(false)
      setCreateForm({ product_id: '', amount: '', currency: 'usd', interval: 'month' })
    },
  })

  const archiveMutation = useMutation({
    mutationFn: adminApi.archiveStripePrice,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe', 'prices'] })
      setArchivingId(null)
    },
  })

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const amountCents = Math.round(parseFloat(createForm.amount) * 100)
    if (isNaN(amountCents) || amountCents < 0) return
    createMutation.mutate({
      product_id: createForm.product_id,
      unit_amount: amountCents,
      currency: createForm.currency,
      interval: createForm.interval,
    })
  }

  const getProductName = (productId: string): string => {
    const product = products?.find((p) => p.id === productId)
    return product?.name ?? productId
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
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          Failed to load prices from Stripe. Check that your API key is valid and that all prices have standard Stripe IDs.
        </AlertDescription>
      </Alert>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Manage pricing for your products.
        </p>
        <Button size="sm" onClick={() => setShowCreateDialog(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Price
        </Button>
      </div>

      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50">
                  <th className="px-4 py-3 text-left font-medium">ID</th>
                  <th className="px-4 py-3 text-left font-medium">Product</th>
                  <th className="px-4 py-3 text-left font-medium">Amount</th>
                  <th className="px-4 py-3 text-left font-medium">Interval</th>
                  <th className="px-4 py-3 text-left font-medium">Status</th>
                  <th className="px-4 py-3 text-right font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {prices?.length === 0 && (
                  <tr>
                    <td colSpan={6} className="px-4 py-8 text-center text-muted-foreground">
                      No prices found. Create one to get started.
                    </td>
                  </tr>
                )}
                {prices?.map((price) => (
                  <tr key={price.id} className="border-b last:border-0">
                    <td className="px-4 py-3">
                      <code className="text-xs font-mono">{price.id}</code>
                    </td>
                    <td className="px-4 py-3">{getProductName(price.product_id)}</td>
                    <td className="px-4 py-3">
                      {price.unit_amount !== null
                        ? formatCurrency(price.unit_amount, price.currency)
                        : '--'}
                    </td>
                    <td className="px-4 py-3 capitalize">
                      {price.recurring_interval ?? 'One-time'}
                    </td>
                    <td className="px-4 py-3">
                      <Badge variant={price.active ? 'default' : 'secondary'}>
                        {price.active ? 'Active' : 'Archived'}
                      </Badge>
                    </td>
                    <td className="px-4 py-3 text-right">
                      {price.active && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          onClick={() => setArchivingId(price.id)}
                        >
                          <Archive className="h-4 w-4" />
                        </Button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Create Price Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={(open) => {
        if (!open) {
          setShowCreateDialog(false)
          setCreateForm({ product_id: '', amount: '', currency: 'usd', interval: 'month' })
          createMutation.reset()
        }
      }}>
        <DialogContent className="sm:max-w-md">
          <form onSubmit={handleCreateSubmit}>
            <DialogHeader>
              <DialogTitle>Create Price</DialogTitle>
              <DialogDescription>Add a new price to a product.</DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label>Product</Label>
                <Select value={createForm.product_id} onValueChange={(v) => setCreateForm({ ...createForm, product_id: v })}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a product" />
                  </SelectTrigger>
                  <SelectContent>
                    {products?.filter((p) => p.active).map((product) => (
                      <SelectItem key={product.id} value={product.id}>
                        {product.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="price_amount">Amount (in dollars)</Label>
                <Input
                  id="price_amount"
                  type="number"
                  step="0.01"
                  min="0.01"
                  required
                  value={createForm.amount}
                  onChange={(e) => setCreateForm({ ...createForm, amount: e.target.value })}
                  placeholder="9.99"
                />
              </div>
              <div className="grid gap-2">
                <Label>Currency</Label>
                <Select value={createForm.currency} onValueChange={(v) => setCreateForm({ ...createForm, currency: v })}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="usd">USD</SelectItem>
                    <SelectItem value="eur">EUR</SelectItem>
                    <SelectItem value="gbp">GBP</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label>Billing Interval</Label>
                <Select value={createForm.interval} onValueChange={(v) => setCreateForm({ ...createForm, interval: v })}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="month">Monthly</SelectItem>
                    <SelectItem value="year">Yearly</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            {createMutation.isError && (
              <Alert variant="destructive" className="mb-4">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {(createMutation.error as unknown as ApiError)?.error?.message || 'Failed to create price'}
                </AlertDescription>
              </Alert>
            )}
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setShowCreateDialog(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={createMutation.isPending || !createForm.product_id}>
                {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Archive Confirm Dialog */}
      <Dialog open={!!archivingId} onOpenChange={(open) => !open && setArchivingId(null)}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Archive Price</DialogTitle>
            <DialogDescription>
              Are you sure you want to archive this price? Existing subscriptions using it will not be affected.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setArchivingId(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={archiveMutation.isPending}
              onClick={() => archivingId && archiveMutation.mutate(archivingId)}
            >
              {archiveMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Archive
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

// ── Webhooks Tab ─────────────────────────────────────────────────────────

function WebhooksTab() {
  const queryClient = useQueryClient()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [createForm, setCreateForm] = useState({
    url: '',
    enabled_events: [] as string[],
  })
  const [deletingId, setDeletingId] = useState<string | null>(null)
  const [createdSecret, setCreatedSecret] = useState<string | null>(null)

  const { data: webhooks, isLoading } = useQuery({
    queryKey: ['admin', 'stripe', 'webhooks'],
    queryFn: adminApi.listStripeWebhooks,
  })

  const createMutation = useMutation({
    mutationFn: adminApi.createStripeWebhook,
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe', 'webhooks'] })
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe'] })
      setShowCreateDialog(false)
      setCreateForm({ url: '', enabled_events: [] })
      if (data.secret) {
        setCreatedSecret(data.secret)
      }
    },
  })

  const deleteMutation = useMutation({
    mutationFn: adminApi.deleteStripeWebhook,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'stripe', 'webhooks'] })
      setDeletingId(null)
    },
  })

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (createForm.enabled_events.length === 0) return
    createMutation.mutate({
      url: createForm.url,
      enabled_events: createForm.enabled_events,
    })
  }

  const toggleEvent = (event: string) => {
    setCreateForm((prev) => ({
      ...prev,
      enabled_events: prev.enabled_events.includes(event)
        ? prev.enabled_events.filter((e) => e !== event)
        : [...prev.enabled_events, event],
    }))
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {createdSecret && (
        <Alert>
          <Key className="h-4 w-4" />
          <AlertTitle>Webhook Secret</AlertTitle>
          <AlertDescription>
            <p className="mb-2">
              Save this signing secret now. It will not be shown again.
            </p>
            <div className="flex items-center gap-2">
              <code className="rounded bg-muted px-2 py-1 text-sm font-mono break-all">
                {createdSecret}
              </code>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 shrink-0"
                onClick={() => copyToClipboard(createdSecret)}
              >
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </AlertDescription>
        </Alert>
      )}

      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Manage webhook endpoints for Stripe events.
        </p>
        <Button size="sm" onClick={() => setShowCreateDialog(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Webhook
        </Button>
      </div>

      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50">
                  <th className="px-4 py-3 text-left font-medium">URL</th>
                  <th className="px-4 py-3 text-left font-medium">Events</th>
                  <th className="px-4 py-3 text-left font-medium">Status</th>
                  <th className="px-4 py-3 text-right font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {webhooks?.length === 0 && (
                  <tr>
                    <td colSpan={4} className="px-4 py-8 text-center text-muted-foreground">
                      No webhook endpoints configured.
                    </td>
                  </tr>
                )}
                {webhooks?.map((webhook) => (
                  <tr key={webhook.id} className="border-b last:border-0">
                    <td className="px-4 py-3">
                      <code className="text-xs font-mono break-all">{webhook.url}</code>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-1">
                        {webhook.enabled_events.slice(0, 3).map((event) => (
                          <Badge key={event} variant="secondary" className="text-xs">
                            {event}
                          </Badge>
                        ))}
                        {webhook.enabled_events.length > 3 && (
                          <Badge variant="secondary" className="text-xs">
                            +{webhook.enabled_events.length - 3} more
                          </Badge>
                        )}
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <Badge variant={webhook.status === 'enabled' ? 'default' : 'secondary'}>
                        {webhook.status}
                      </Badge>
                    </td>
                    <td className="px-4 py-3 text-right">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-destructive"
                        onClick={() => setDeletingId(webhook.id)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Create Webhook Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={(open) => {
        if (!open) {
          setShowCreateDialog(false)
          setCreateForm({ url: '', enabled_events: [] })
          createMutation.reset()
        }
      }}>
        <DialogContent className="sm:max-w-md">
          <form onSubmit={handleCreateSubmit}>
            <DialogHeader>
              <DialogTitle>Create Webhook Endpoint</DialogTitle>
              <DialogDescription>
                Configure a URL to receive Stripe event notifications.
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="webhook_url">Endpoint URL</Label>
                <Input
                  id="webhook_url"
                  type="url"
                  required
                  value={createForm.url}
                  onChange={(e) => setCreateForm({ ...createForm, url: e.target.value })}
                  placeholder="https://example.com/v1/webhooks/stripe"
                />
              </div>
              <div className="grid gap-2">
                <Label>Events</Label>
                <div className="space-y-2 rounded-md border p-3">
                  {WEBHOOK_EVENTS.map((event) => (
                    <label key={event} className="flex items-center gap-2 text-sm cursor-pointer">
                      <input
                        type="checkbox"
                        checked={createForm.enabled_events.includes(event)}
                        onChange={() => toggleEvent(event)}
                        className="rounded border-input"
                      />
                      <code className="text-xs">{event}</code>
                    </label>
                  ))}
                </div>
                <div className="flex justify-end">
                  <Button
                    type="button"
                    variant="link"
                    size="sm"
                    className="h-auto p-0 text-xs"
                    onClick={() =>
                      setCreateForm({ ...createForm, enabled_events: [...WEBHOOK_EVENTS] })
                    }
                  >
                    Select all
                  </Button>
                </div>
              </div>
            </div>
            {createMutation.isError && (
              <Alert variant="destructive" className="mb-4">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {(createMutation.error as unknown as ApiError)?.error?.message || 'Failed to create webhook'}
                </AlertDescription>
              </Alert>
            )}
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setShowCreateDialog(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={createMutation.isPending || createForm.enabled_events.length === 0}>
                {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete Confirm Dialog */}
      <Dialog open={!!deletingId} onOpenChange={(open) => !open && setDeletingId(null)}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Delete Webhook</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this webhook endpoint? Stripe will stop sending events to this URL.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeletingId(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={deleteMutation.isPending}
              onClick={() => deletingId && deleteMutation.mutate(deletingId)}
            >
              {deleteMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

// ── Main Page ────────────────────────────────────────────────────────────

export function AdminStripePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Stripe</h1>
        <p className="text-muted-foreground">
          Manage your Stripe payment integration, products, prices, and webhook endpoints.
        </p>
      </div>

      <Tabs defaultValue="keys" className="space-y-4">
        <TabsList>
          <TabsTrigger value="keys">API Keys</TabsTrigger>
          <TabsTrigger value="products">Products</TabsTrigger>
          <TabsTrigger value="prices">Prices</TabsTrigger>
          <TabsTrigger value="webhooks">Webhooks</TabsTrigger>
        </TabsList>

        <TabsContent value="keys">
          <ApiKeysTab />
        </TabsContent>

        <TabsContent value="products">
          <ProductsTab />
        </TabsContent>

        <TabsContent value="prices">
          <PricesTab />
        </TabsContent>

        <TabsContent value="webhooks">
          <WebhooksTab />
        </TabsContent>
      </Tabs>
    </div>
  )
}
