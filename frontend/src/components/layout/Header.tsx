import { Link } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { Button } from '@/components/ui/button'

export function Header() {
  const { isAuthenticated, user, logout } = useAuthStore()

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container flex h-16 items-center justify-between">
        <div className="flex items-center gap-6">
          <Link to="/" className="flex items-center space-x-2">
            <span className="text-2xl font-bold text-primary">a8n</span>
            <span className="text-2xl font-light">.tools</span>
          </Link>
          <nav className="hidden md:flex items-center gap-6">
            <Link
              to="/pricing"
              className="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
            >
              Pricing
            </Link>
          </nav>
        </div>

        <div className="flex items-center gap-4">
          {isAuthenticated ? (
            <>
              <Link to="/dashboard">
                <Button variant="ghost" size="sm">
                  Dashboard
                </Button>
              </Link>
              {user?.role === 'admin' && (
                <Link to="/admin">
                  <Button variant="ghost" size="sm">
                    Admin
                  </Button>
                </Link>
              )}
              <Button variant="outline" size="sm" onClick={() => logout()}>
                Logout
              </Button>
            </>
          ) : (
            <>
              <Link to="/login">
                <Button variant="ghost" size="sm">
                  Login
                </Button>
              </Link>
              <Link to="/register">
                <Button size="sm">Get Started</Button>
              </Link>
            </>
          )}
        </div>
      </div>
    </header>
  )
}
