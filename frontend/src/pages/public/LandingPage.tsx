import { useMemo } from 'react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useInView } from '@/hooks/useInView'
import { useApplications } from '@/hooks/useApplications'
import { useAuthStore } from '@/stores/authStore'
import { config } from '@/config'
import { getAppGradient } from '@/lib/utils'

const heroLines = [
  { plain: 'All access.', gradient: 'No clock.' },
  { plain: 'All in.', gradient: 'All yours.' },
  { plain: 'Price locked.', gradient: 'Tools stocked.' },
  { plain: 'Locked in.', gradient: 'Lights on.' },
  { plain: 'Subscribed once', gradient: 'Sorted Forever.' },
  { plain: 'One Price', gradient: 'For life.' },
  { plain: 'Open source', gradient: 'For life.' },
]

const features = [
  {
    icon: 'fa-solid fa-bolt',
    title: 'Blazing Fast',
    description: 'Written in Rust. No garbage collector, no runtime overhead. Just raw speed.',
    gradient: 'from-primary to-primary/60',
    borderGradient: 'from-primary/50 via-primary/20 to-transparent',
  },
  {
    icon: 'fa-solid fa-shield',
    title: 'Secure by Default',
    description: 'Memory-safe, type-safe, battle-tested. Sleep well at night.',
    gradient: 'from-indigo-500 to-indigo-500/60',
    borderGradient: 'from-indigo-500/50 via-indigo-500/20 to-transparent',
  },
  {
    icon: 'fa-solid fa-dollar-sign',
    title: '$3/month. Forever.',
    description: 'One price, locked for life. No tiers. No surprises. No "enterprise" upsells.',
    gradient: 'from-teal-500 to-teal-500/60',
    borderGradient: 'from-teal-500/50 via-teal-500/20 to-transparent',
  },
]

export function LandingPage() {
  const hero = useMemo(() => heroLines[Math.floor(Math.random() * heroLines.length)], [])
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated)
  const { applications } = useApplications()
  const features$ = useInView(0.1)
  const apps$ = useInView(0.1)
  const cta$ = useInView(0.15)

  const getAppUrl = (app: { subdomain: string | null; slug: string }) => {
    const subdomain = app.subdomain || app.slug
    return config.appDomain ? `https://${subdomain}.${config.appDomain}` : '#'
  }

  return (
    <div>
      {/* Hero Section */}
      <section className="relative overflow-hidden py-20 md:py-32">
        <div className="container relative flex flex-col items-center text-center">
          <div className="mb-6 inline-flex items-center gap-2 rounded-full bg-zinc-800 px-4 py-1.5 text-sm text-zinc-300 dark:bg-zinc-800 dark:text-zinc-400 hero-fade-up">
            <i className="fa-solid fa-terminal text-[0.875rem]" />
            Open source. Rust-powered. Fully managed.
          </div>
          <h1 className="text-4xl font-bold tracking-tight sm:text-5xl md:text-6xl lg:text-7xl hero-fade-up-1">
            {hero.plain}{' '}
            <span className="text-gradient bg-gradient-to-r from-primary via-indigo-500 to-teal-400 hero-gradient-shift">
              {hero.gradient}
            </span>
          </h1>
          <p className="mt-6 max-w-2xl text-lg text-muted-foreground md:text-xl hero-fade-up-2">
            Tools that run themselves so you can focus on what you're actually building.
            One subscription.{' '}
            <span className="font-semibold text-foreground">$3/month</span>, locked forever.
          </p>
          <div className="mt-10 flex flex-col gap-4 sm:flex-row hero-fade-up-3">
            <Link to={isAuthenticated ? '/membership' : '/register'}>
              <Button size="lg" className="w-full sm:w-auto gap-2 bg-gradient-to-r from-primary to-indigo-500 hover:from-primary/90 hover:to-indigo-500/90 border-0 text-white shadow-lg shadow-primary/25">
                {isAuthenticated ? 'Go to Membership' : 'Get Started'} <i className="fa-solid fa-arrow-right text-[1rem]" />
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
        <div
          ref={features$.ref}
          className={`container relative scroll-fade-up ${features$.inView ? 'in-view' : ''}`}
        >
          <h2 className="text-center text-3xl font-bold">No ops. No overhead. No nonsense.</h2>
          <p className="mx-auto mt-4 max-w-2xl text-center text-muted-foreground">
            We handle hosting, updates, and uptime. You get tools that just work.
          </p>
          <div className={`mt-12 grid gap-8 md:grid-cols-3 scroll-fade-up-child ${features$.inView ? 'in-view' : ''}`}>
            {features.map((feature) => (
              <div key={feature.title} className="group relative rounded-xl">
                {/* Gradient border effect */}
                <div className={`absolute -inset-px rounded-xl bg-gradient-to-b ${feature.borderGradient} opacity-0 transition-opacity group-hover:opacity-100`} />
                <Card className="relative border-0 bg-card/80 backdrop-blur-sm">
                  <CardHeader>
                    <div className={`flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-br ${feature.gradient}`}>
                      <i className={`${feature.icon} text-xl text-white`} />
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
        <div
          ref={apps$.ref}
          className={`container relative scroll-fade-up ${apps$.inView ? 'in-view' : ''}`}
        >
          <h2 className="text-center text-3xl font-bold">The toolkit</h2>
          <p className="mx-auto mt-4 max-w-2xl text-center text-muted-foreground">
            All included. More shipping soon.
          </p>
          <div className={`mt-12 grid gap-8 md:grid-cols-2 max-w-3xl mx-auto scroll-fade-up-child ${apps$.inView ? 'in-view' : ''}`}>
            {applications.map((app, index) => {
              const gradient = getAppGradient(index)
              return (
                <Card key={app.id} className="transition-all hover:shadow-lg hover:shadow-indigo-500/5 border-border/50">
                  <CardHeader>
                    <div className="flex items-center gap-4">
                      <div className={`flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-br ${gradient}`}>
                        {app.icon_url ? (
                          <img src={app.icon_url} alt={app.display_name} className="h-6 w-6" />
                        ) : (
                          <i className="fa-solid fa-cube text-xl text-white" />
                        )}
                      </div>
                      <div>
                        <CardTitle>{app.display_name}</CardTitle>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <CardDescription className="text-base">
                      {app.description}
                    </CardDescription>
                    <a
                      href={getAppUrl(app)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className={`mt-4 inline-flex items-center gap-1 text-sm text-gradient bg-gradient-to-r ${gradient} font-medium hover:underline`}
                    >
                      Learn more <i className="fa-solid fa-arrow-right text-xs text-indigo-500" />
                    </a>
                  </CardContent>
                </Card>
              )
            })}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative overflow-hidden border-t border-border/50 py-20">
        <div className="absolute inset-0 bg-gradient-to-br from-indigo-600 via-primary to-teal-500" />
        <div
          ref={cta$.ref}
          className={`container relative text-center scroll-fade-up ${cta$.inView ? 'in-view' : ''}`}
        >
          <h2 className="text-3xl font-bold text-white">
            Stop configuring. Start building.
          </h2>
          <p className="mx-auto mt-4 max-w-2xl text-white/80">
            Lock in $3/month. Get every tool, current and future. Cancel anytime.
          </p>
          <div className="mt-10 flex justify-center gap-4">
            <Link to={isAuthenticated ? '/membership' : '/register'}>
              <Button size="lg" variant="secondary" className="gap-2 shadow-lg">
                {isAuthenticated ? 'Go to Membership' : 'Create Account'} <i className="fa-solid fa-arrow-right text-[1rem]" />
              </Button>
            </Link>
          </div>
        </div>
      </section>
    </div>
  )
}
