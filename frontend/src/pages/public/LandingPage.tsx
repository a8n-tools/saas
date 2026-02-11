import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Link2, Bookmark, Zap, Shield, DollarSign } from 'lucide-react'

const features = [
  {
    icon: Zap,
    title: 'Lightning Fast',
    description: 'All our apps are built with Rust for maximum performance.',
  },
  {
    icon: Shield,
    title: 'Secure by Default',
    description: 'Enterprise-grade security with no compromises.',
  },
  {
    icon: DollarSign,
    title: 'Fixed Price Forever',
    description: 'Lock in $3/month for life. Early adopters win.',
  },
]

const apps = [
  {
    icon: Link2,
    name: 'RUS',
    description: 'Rust URL Shortener with QR code generation.',
    url: 'https://rus.a8n.tools',
  },
  {
    icon: Bookmark,
    name: 'Rusty Links',
    description: 'Beautiful bookmark management for power users.',
    url: 'https://rustylinks.a8n.tools',
  },
]

export function LandingPage() {
  return (
    <div>
      {/* Hero Section */}
      <section className="py-20 md:py-32">
        <div className="container flex flex-col items-center text-center">
          <h1 className="text-4xl font-bold tracking-tight sm:text-5xl md:text-6xl lg:text-7xl">
            Developer tools,{' '}
            <span className="text-primary">automated.</span>
          </h1>
          <p className="mt-6 max-w-2xl text-lg text-muted-foreground md:text-xl">
            Open source tools, managed for you. All apps for{' '}
            <span className="font-semibold text-foreground">$3/month</span>.
            Lock in your price forever.
          </p>
          <div className="mt-10 flex flex-col gap-4 sm:flex-row">
            <Link to="/register">
              <Button size="lg" className="w-full sm:w-auto">
                Get Started
              </Button>
            </Link>
            <Link to="/pricing">
              <Button size="lg" variant="outline" className="w-full sm:w-auto">
                View Pricing
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="border-t bg-muted/50 py-20">
        <div className="container">
          <h2 className="text-center text-3xl font-bold">Why a8n.tools?</h2>
          <p className="mx-auto mt-4 max-w-2xl text-center text-muted-foreground">
            We handle the infrastructure so you can focus on what matters.
          </p>
          <div className="mt-12 grid gap-8 md:grid-cols-3">
            {features.map((feature) => (
              <Card key={feature.title} className="border-0 bg-background">
                <CardHeader>
                  <feature.icon className="h-10 w-10 text-primary" />
                  <CardTitle className="mt-4">{feature.title}</CardTitle>
                </CardHeader>
                <CardContent>
                  <CardDescription className="text-base">
                    {feature.description}
                  </CardDescription>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* Apps Section */}
      <section className="py-20">
        <div className="container">
          <h2 className="text-center text-3xl font-bold">Our Applications</h2>
          <p className="mx-auto mt-4 max-w-2xl text-center text-muted-foreground">
            All included with your membership. More coming soon.
          </p>
          <div className="mt-12 grid gap-8 md:grid-cols-2 max-w-3xl mx-auto">
            {apps.map((app) => (
              <Card key={app.name}>
                <CardHeader>
                  <div className="flex items-center gap-4">
                    <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                      <app.icon className="h-6 w-6 text-primary" />
                    </div>
                    <div>
                      <CardTitle>{app.name}</CardTitle>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <CardDescription className="text-base">
                    {app.description}
                  </CardDescription>
                  <a
                    href={app.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="mt-4 inline-flex items-center text-sm text-primary hover:underline"
                  >
                    Learn more &rarr;
                  </a>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* Pricing CTA Section */}
      <section className="border-t bg-primary py-20">
        <div className="container text-center">
          <h2 className="text-3xl font-bold text-primary-foreground">
            Ready to get started?
          </h2>
          <p className="mx-auto mt-4 max-w-2xl text-primary-foreground/80">
            Join today and lock in your $3/month price forever.
          </p>
          <div className="mt-10 flex justify-center gap-4">
            <Link to="/register">
              <Button size="lg" variant="secondary">
                Create Account
              </Button>
            </Link>
          </div>
        </div>
      </section>
    </div>
  )
}
