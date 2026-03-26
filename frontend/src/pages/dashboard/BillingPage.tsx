import { useQuery } from '@tanstack/react-query'
import { ExternalLink, FileText } from 'lucide-react'
import { billingApi } from '@/api'
import type { StripeInvoice } from '@/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString('en-US', {
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
  const {
    data: invoices,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['billing-invoices'],
    queryFn: () => billingApi.listInvoices(),
  })

  const handleViewInvoice = (invoice: StripeInvoice) => {
    const url = invoice.hosted_invoice_url || invoice.invoice_pdf
    if (url) {
      window.open(url, '_blank', 'noopener,noreferrer')
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
          {isLoading && (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary" />
            </div>
          )}

          {error && (
            <p className="text-sm text-destructive py-4">Failed to load invoices.</p>
          )}

          {!isLoading && !error && (!invoices || invoices.length === 0) && (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <FileText className="h-10 w-10 text-muted-foreground mb-3" />
              <p className="text-muted-foreground">No invoices yet.</p>
            </div>
          )}

          {!isLoading && invoices && invoices.length > 0 && (
            <div className="divide-y">
              {invoices.map((invoice) => (
                <div
                  key={invoice.id}
                  className="flex items-center justify-between py-4"
                >
                  <div className="space-y-1">
                    {invoice.number && (
                      <p className="font-medium text-sm">{invoice.number}</p>
                    )}
                    {invoice.description && (
                      <p className="text-sm text-muted-foreground">{invoice.description}</p>
                    )}
                    <p className="text-xs text-muted-foreground">{formatDate(invoice.created)}</p>
                  </div>
                  <div className="flex items-center gap-4">
                    <span className="text-sm font-medium">
                      {formatAmount(invoice.amount_paid, invoice.currency)}
                    </span>
                    {(invoice.hosted_invoice_url || invoice.invoice_pdf) && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleViewInvoice(invoice)}
                      >
                        <ExternalLink className="h-4 w-4 mr-1" />
                        View
                      </Button>
                    )}
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
