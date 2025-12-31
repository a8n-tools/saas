import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { CreditCard, ArrowLeft } from 'lucide-react'

export function SubscriptionRequiredPage() {
  return (
    <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center px-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
            <CreditCard className="h-8 w-8 text-primary" />
          </div>
          <CardTitle className="text-2xl">Subscription Required</CardTitle>
          <CardDescription>
            You need an active subscription to access this content.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-center text-muted-foreground">
            Subscribe to get access to all applications for just $3/month.
            Lock in your price forever.
          </p>
          <div className="flex flex-col gap-4">
            <Link to="/subscription">
              <Button className="w-full">Subscribe Now</Button>
            </Link>
            <Link to="/dashboard">
              <Button variant="outline" className="w-full">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Back to Dashboard
              </Button>
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
