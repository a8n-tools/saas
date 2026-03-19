import { Link, useLocation } from 'react-router-dom'
import { SmilePlus } from 'lucide-react'

export function FeedbackLauncher() {
  const location = useLocation()

  if (location.pathname === '/feedback' || location.pathname.startsWith('/admin')) {
    return null
  }

  return (
    <div className="pointer-events-none fixed bottom-4 right-4 z-40 sm:bottom-6 sm:right-6">
      <Link
        to="/feedback"
        state={{ fromPath: location.pathname + location.search }}
        aria-label="Open feedback page"
        className="pointer-events-auto group flex h-14 w-[60px] items-center overflow-hidden rounded-2xl border border-border/70 bg-background/85 text-primary shadow-xl shadow-primary/10 backdrop-blur-md transition-all duration-300 hover:w-[204px] hover:border-primary/50 hover:bg-background dark:bg-card/90 sm:h-16 sm:w-16 sm:hover:w-[214px]"
      >
        <span className="relative inline-flex h-14 w-[60px] shrink-0 items-center justify-center rounded-2xl sm:h-16 sm:w-16">
          <span className="absolute inset-0 rounded-2xl bg-gradient-to-br from-primary/18 via-indigo-500/12 to-teal-500/18 opacity-80" />
          <SmilePlus className="feedback-launcher__icon-bounce relative z-10 h-7 w-7 sm:h-8 sm:w-8" />
        </span>
        <span className="max-w-0 whitespace-nowrap pl-0 pr-0 text-sm font-medium text-foreground opacity-0 transition-all duration-300 group-hover:max-w-[130px] group-hover:pl-4 group-hover:pr-4 group-hover:opacity-100">
          Have feedback?
        </span>
      </Link>
    </div>
  )
}
