import { useMemo } from 'react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Link2, Bookmark, Zap, Shield, DollarSign, Terminal, ArrowRight } from 'lucide-react'

const heroLines = [
  { plain: 'All access.', gradient: 'No clock.' },
  { plain: 'All in.', gradient: 'All yours.' },
  { plain: 'Price locked.', gradient: 'Tools stocked.' },
  { plain: 'Locked in.', gradient: 'Lights on' },
  { plain: 'Subscribed once', gradient: 'Sorted Forever.' },
]

const features = [
  {
    icon: Zap,
    title: 'Blazing Fast',
    description: 'Written in Rust. No garbage collector, no runtime overhead. Just raw speed.',
    gradient: 'from-primary to-primary/60',
    borderGradient: 'from-primary/50 via-primary/20 to-transparent',
  },
  {
    icon: Shield,
    title: 'Secure by Default',
    description: 'Memory-safe, type-safe, battle-tested. Sleep well at night.',
    gradient: 'from-indigo-500 to-indigo-500/60',
    borderGradient: 'from-indigo-500/50 via-indigo-500/20 to-transparent',
  },
  {
    icon: DollarSign,
    title: '$3/month. Forever.',
    description: 'One price, locked for life. No tiers. No surprises. No "enterprise" upsells.',
    gradient: 'from-teal-500 to-teal-500/60',
    borderGradient: 'from-teal-500/50 via-teal-500/20 to-transparent',
  },
]

const apps = [
  {
    icon: Link2,
    name: 'RUS',
    description: 'URL shortener with QR generation. Self-hostable, API-first, zero bloat.',
    url: 'https://rus.a8n.tools',
    gradient: 'from-indigo-500 to-primary',
    borderColor: 'border-indigo-500/20 hover:border-indigo-500/40',
  },
  {
    icon: Bookmark,
    name: 'Rusty Links',
    description: 'Bookmark manager for people with too many tabs. Tag, search, organize.',
    url: 'https://rustylinks.a8n.tools',
    gradient: 'from-teal-500 to-indigo-500',
    borderColor: 'border-teal-500/20 hover:border-teal-500/40',
  },
]

export function LandingPage() {
  const hero = useMemo(() => heroLines[Math.floor(Math.random() * heroLines.length)], [])

  return (
    <div>
      {/* Hero Section */}
      <section className="relative overflow-hidden py-20 md:py-32">
        {/* Gradient orbs */}
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute -top-24 left-1/4 h-96 w-96 rounded-full bg-primary/15 blur-3xl" />
          <div className="absolute -top-24 right-1/4 h-96 w-96 rounded-full bg-indigo-500/15 blur-3xl" />
          <div className="absolute top-32 left-1/2 h-64 w-64 -translate-x-1/2 rounded-full bg-teal-500/10 blur-3xl" />
        </div>

        <div className="container relative flex flex-col items-center text-center">
          <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-indigo-300/30 bg-indigo-500/10 px-4 py-1.5 text-sm text-indigo-600 backdrop-blur-sm dark:border-indigo-500/20 dark:bg-indigo-500/10 dark:text-indigo-400">
            <Terminal className="h-3.5 w-3.5" />
            Open source. Rust-powered. Fully managed.
          </div>
          <h1 className="text-4xl font-bold tracking-tight sm:text-5xl md:text-6xl lg:text-7xl">
            {hero.plain}{' '}
            <span className="text-gradient bg-gradient-to-r from-primary via-indigo-500 to-teal-400">
              {hero.gradient}
            </span>
          </h1>
          <p className="mt-6 max-w-2xl text-lg text-muted-foreground md:text-xl">
            Tools that run themselves so you can focus on what you're actually building.
            One subscription.{' '}
            <span className="font-semibold text-foreground">$3/month</span>, locked forever.
          </p>
          <div className="mt-10 flex flex-col gap-4 sm:flex-row">
            <Link to="/register">
              <Button size="lg" className="w-full sm:w-auto gap-2 bg-gradient-to-r from-primary to-indigo-500 hover:from-primary/90 hover:to-indigo-500/90 border-0 text-white shadow-lg shadow-primary/25">
                Get Started <ArrowRight className="h-4 w-4" />
              </Button>
            </Link>
            <Link to="/pricing">
              <Button size="lg" variant="outline" className="w-full sm:w-auto border-indigo-300/30 text-indigo-600 hover:bg-indigo-500/10 dark:border-indigo-500/30 dark:text-indigo-400 dark:hover:bg-indigo-500/10">
                View Pricing
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="relative border-t border-border/50 py-20">
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-indigo-500/[0.03] via-transparent to-teal-500/[0.03]" />
        <div className="container relative">
          <h2 className="text-center text-3xl font-bold">No ops. No overhead. No nonsense.</h2>
          <p className="mx-auto mt-4 max-w-2xl text-center text-muted-foreground">
            We handle hosting, updates, and uptime. You get tools that just work.
          </p>
          <div className="mt-12 grid gap-8 md:grid-cols-3">
            {features.map((feature) => (
              <div key={feature.title} className="group relative rounded-xl">
                {/* Gradient border effect */}
                <div className={`absolute -inset-px rounded-xl bg-gradient-to-b ${feature.borderGradient} opacity-0 transition-opacity group-hover:opacity-100`} />
                <Card className="relative border-0 bg-card/80 backdrop-blur-sm">
                  <CardHeader>
                    <div className={`flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-br ${feature.gradient}`}>
                      <feature.icon className="h-6 w-6 text-white" />
                    </div>
                    <CardTitle className="mt-4">{feature.title}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <CardDescription className="text-base">
                      {feature.description}
                    </CardDescription>
                  </CardContent>
                </Card>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Apps Section */}
      <section className="relative py-20">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute bottom-0 left-1/3 h-64 w-64 rounded-full bg-teal-500/10 blur-3xl" />
          <div className="absolute bottom-0 right-1/3 h-64 w-64 rounded-full bg-indigo-500/10 blur-3xl" />
        </div>
        <div className="container relative">
          <h2 className="text-center text-3xl font-bold">The toolkit</h2>
          <p className="mx-auto mt-4 max-w-2xl text-center text-muted-foreground">
            All included. More shipping soon.
          </p>
          <div className="mt-12 grid gap-8 md:grid-cols-2 max-w-3xl mx-auto">
            {apps.map((app) => (
              <Card key={app.name} className={`transition-all hover:shadow-lg hover:shadow-indigo-500/5 ${app.borderColor}`}>
                <CardHeader>
                  <div className="flex items-center gap-4">
                    <div className={`flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-br ${app.gradient}`}>
                      <app.icon className="h-6 w-6 text-white" />
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
                    className={`mt-4 inline-flex items-center gap-1 text-sm text-gradient bg-gradient-to-r ${app.gradient} font-medium hover:underline`}
                  >
                    Learn more <ArrowRight className={`h-3.5 w-3.5 text-indigo-500`} />
                  </a>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative overflow-hidden border-t border-border/50 py-20">
        <div className="absolute inset-0 bg-gradient-to-br from-indigo-600 via-primary to-teal-500" />
        {/* Subtle noise/glow overlay */}
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute -top-20 left-1/4 h-64 w-64 rounded-full bg-white/10 blur-3xl" />
          <div className="absolute -bottom-20 right-1/4 h-64 w-64 rounded-full bg-white/10 blur-3xl" />
        </div>
        <div className="container relative text-center">
          <h2 className="text-3xl font-bold text-white">
            Stop configuring. Start building.
          </h2>
          <p className="mx-auto mt-4 max-w-2xl text-white/80">
            Lock in $3/month. Get every tool, current and future. Cancel anytime.
          </p>
          <div className="mt-10 flex justify-center gap-4">
            <Link to="/register">
              <Button size="lg" variant="secondary" className="gap-2 shadow-lg">
                Create Account <ArrowRight className="h-4 w-4" />
              </Button>
            </Link>
          </div>
        </div>
      </section>
    </div>
  )
}
