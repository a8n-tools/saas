import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Check } from 'lucide-react'

const personalFeatures = [
  'Access to all current applications',
  'Access to all future applications',
  'Community support',
  'Price locked for life',
  'No hidden fees',
  'Cancel anytime',
]

const businessFeatures = [
  'Everything in Personal',
  'Priority support',
  'Team management (coming soon)',
  'Usage analytics (coming soon)',
  'Invoice billing',
  'Dedicated account manager',
]

const showBusinessPricing = import.meta.env.VITE_SHOW_BUSINESS_PRICING === 'true'

export function PricingPage() {
  return (
    <div className="py-20">
      <div className="container">
        <div className="text-center">
          <h1 className="text-4xl font-bold">Simple, Transparent Pricing</h1>
          <p className="mt-4 text-lg text-muted-foreground">
            Choose the plan that fits your needs. All prices locked for life.
          </p>
        </div>

        <div className={`mt-16 grid gap-8 ${showBusinessPricing ? 'md:grid-cols-2 max-w-4xl' : 'max-w-md'} mx-auto`}>
          {/* Personal Plan */}
          <Card className="w-full border-primary">
            <CardHeader className="text-center">
              <CardTitle className="text-2xl">Personal</CardTitle>
              <CardDescription>
                Perfect for individual developers
              </CardDescription>
              <div className="mt-6">
                <span className="text-5xl font-bold">$3</span>
                <span className="text-muted-foreground">/month</span>
              </div>
              <p className="mt-2 text-sm text-muted-foreground">
                Price locked for life when you join
              </p>
            </CardHeader>
            <CardContent>
              <ul className="space-y-4">
                {personalFeatures.map((feature) => (
                  <li key={feature} className="flex items-center gap-3">
                    <Check className="h-5 w-5 text-primary flex-shrink-0" />
                    <span>{feature}</span>
                  </li>
                ))}
              </ul>
              <Link to="/register?tier=personal" className="mt-8 block">
                <Button className="w-full" size="lg">
                  Get Started
                </Button>
              </Link>
              <p className="mt-4 text-center text-sm text-muted-foreground">
                30-day grace period if payment fails
              </p>
            </CardContent>
          </Card>

          {/* Business Plan */}
          {showBusinessPricing && (
            <Card className="w-full border-2 border-primary relative">
              <Badge className="absolute -top-3 left-1/2 -translate-x-1/2">
                For Teams
              </Badge>
              <CardHeader className="text-center">
                <CardTitle className="text-2xl">Business</CardTitle>
                <CardDescription>
                  Built for teams and organizations
                </CardDescription>
                <div className="mt-6">
                  <span className="text-5xl font-bold">$15</span>
                  <span className="text-muted-foreground">/month</span>
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  Price locked for life when you join
                </p>
              </CardHeader>
              <CardContent>
                <ul className="space-y-4">
                  {businessFeatures.map((feature) => (
                    <li key={feature} className="flex items-center gap-3">
                      <Check className="h-5 w-5 text-primary flex-shrink-0" />
                      <span>{feature}</span>
                    </li>
                  ))}
                </ul>
                <Link to="/register?tier=business" className="mt-8 block">
                  <Button className="w-full" size="lg">
                    Get Started
                  </Button>
                </Link>
                <p className="mt-4 text-center text-sm text-muted-foreground">
                  30-day grace period if payment fails
                </p>
              </CardContent>
            </Card>
          )}
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
                the same rate you signed up for{showBusinessPricing ? ' ($3 or $15/month)' : ''}.
              </p>
            </div>
            <div>
              <h3 className="font-semibold">What if I cancel and rejoin?</h3>
              <p className="mt-2 text-muted-foreground">
                As long as you had an active membership, your locked price
                will be honored when you rejoin. We keep track of your
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
                membership. After that, you can cancel anytime but won't
                receive a refund for the current period.
              </p>
            </div>
            {showBusinessPricing && (
              <div>
                <h3 className="font-semibold">What's the difference between Personal and Business?</h3>
                <p className="mt-2 text-muted-foreground">
                  Both plans provide access to all applications. Business includes
                  priority support, invoice billing, and upcoming team features
                  like team management and usage analytics.
                </p>
              </div>
            )}
            {showBusinessPricing && (
              <div>
                <h3 className="font-semibold">Can I upgrade from Personal to Business?</h3>
                <p className="mt-2 text-muted-foreground">
                  Yes! You can upgrade at any time. The price difference will be
                  prorated for the remainder of your billing period.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
