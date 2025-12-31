import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Check } from 'lucide-react'

const features = [
  'Access to all current applications',
  'Access to all future applications',
  'Priority support',
  'Price locked for life',
  'No hidden fees',
  'Cancel anytime',
]

export function PricingPage() {
  return (
    <div className="py-20">
      <div className="container">
        <div className="text-center">
          <h1 className="text-4xl font-bold">Simple, Transparent Pricing</h1>
          <p className="mt-4 text-lg text-muted-foreground">
            One price. All apps. Forever.
          </p>
        </div>

        <div className="mt-16 flex justify-center">
          <Card className="w-full max-w-md border-primary">
            <CardHeader className="text-center">
              <CardTitle className="text-2xl">All Access</CardTitle>
              <CardDescription>
                Everything you need, nothing you don't
              </CardDescription>
              <div className="mt-6">
                <span className="text-5xl font-bold">$3</span>
                <span className="text-muted-foreground">/month</span>
              </div>
              <p className="mt-2 text-sm text-muted-foreground">
                Price locked for life when you subscribe
              </p>
            </CardHeader>
            <CardContent>
              <ul className="space-y-4">
                {features.map((feature) => (
                  <li key={feature} className="flex items-center gap-3">
                    <Check className="h-5 w-5 text-primary flex-shrink-0" />
                    <span>{feature}</span>
                  </li>
                ))}
              </ul>
              <Link to="/register" className="mt-8 block">
                <Button className="w-full" size="lg">
                  Get Started
                </Button>
              </Link>
              <p className="mt-4 text-center text-sm text-muted-foreground">
                30-day grace period if payment fails
              </p>
            </CardContent>
          </Card>
        </div>

        <div className="mt-20">
          <h2 className="text-center text-2xl font-bold">
            Frequently Asked Questions
          </h2>
          <div className="mt-10 grid gap-8 md:grid-cols-2 max-w-4xl mx-auto">
            <div>
              <h3 className="font-semibold">What does "price locked for life" mean?</h3>
              <p className="mt-2 text-muted-foreground">
                When you subscribe, your monthly price is locked in forever.
                Even if we raise prices for new subscribers, you'll always pay
                the same $3/month.
              </p>
            </div>
            <div>
              <h3 className="font-semibold">What if I cancel and resubscribe?</h3>
              <p className="mt-2 text-muted-foreground">
                As long as you had an active subscription, your locked price
                will be honored when you resubscribe. We keep track of your
                original price.
              </p>
            </div>
            <div>
              <h3 className="font-semibold">What happens if my payment fails?</h3>
              <p className="mt-2 text-muted-foreground">
                You have a 30-day grace period to update your payment method.
                During this time, you'll still have access to all applications.
              </p>
            </div>
            <div>
              <h3 className="font-semibold">Do you offer refunds?</h3>
              <p className="mt-2 text-muted-foreground">
                We offer refunds within the first 7 days of your initial
                subscription. After that, you can cancel anytime but won't
                receive a refund for the current period.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
