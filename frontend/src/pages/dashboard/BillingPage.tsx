import { useEffect, useState } from 'react'
import { Download, FileText } from 'lucide-react'
import { billingApi } from '@/api'
import type { Invoice } from '@/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}

function formatAmount(cents: number, currency: string): string {
  if (cents === 0) return 'Free'
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency.toUpperCase(),
  }).format(cents / 100)
}

export function BillingPage() {
  const [invoices, setInvoices] = useState<Invoice[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [downloading, setDownloading] = useState<string | null>(null)

  useEffect(() => {
    billingApi
      .listInvoices()
      .then(setInvoices)
      .catch(() => setError('Failed to load invoices.'))
      .finally(() => setLoading(false))
  }, [])

  const handleDownload = async (invoice: Invoice) => {
    setDownloading(invoice.id)
    try {
      await billingApi.downloadInvoice(invoice.id, invoice.invoice_number)
    } catch {
      setError('Failed to download invoice.')
    } finally {
      setDownloading(null)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Billing</h1>
        <p className="text-muted-foreground">View and download your invoices.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Invoices</CardTitle>
          <CardDescription>Your billing history</CardDescription>
        </CardHeader>
        <CardContent>
          {loading && (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary" />
            </div>
          )}

          {error && (
            <p className="text-sm text-destructive py-4">{error}</p>
          )}

          {!loading && !error && invoices.length === 0 && (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <FileText className="h-10 w-10 text-muted-foreground mb-3" />
              <p className="text-muted-foreground">No invoices yet.</p>
            </div>
          )}

          {!loading && invoices.length > 0 && (
            <div className="divide-y">
              {invoices.map((invoice) => (
                <div
                  key={invoice.id}
                  className="flex items-center justify-between py-4"
                >
                  <div className="space-y-1">
                    <p className="font-medium text-sm">{invoice.invoice_number}</p>
                    <p className="text-sm text-muted-foreground">{invoice.description}</p>
                    <p className="text-xs text-muted-foreground">{formatDate(invoice.created_at)}</p>
                  </div>
                  <div className="flex items-center gap-4">
                    <span className="text-sm font-medium">
                      {formatAmount(invoice.amount_cents, invoice.currency)}
                    </span>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDownload(invoice)}
                      disabled={downloading === invoice.id}
                    >
                      <Download className="h-4 w-4 mr-1" />
                      {downloading === invoice.id ? 'Downloading…' : 'PDF'}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
