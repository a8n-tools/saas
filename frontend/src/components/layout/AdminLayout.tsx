import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { useAuthStore } from '@/stores/authStore'
import { Button } from '@/components/ui/button'
import { LogOut, User, ArrowLeft } from 'lucide-react'
import { Link } from 'react-router-dom'

export function AdminLayout() {
  const { user, logout } = useAuthStore()

  return (
    <div className="flex min-h-screen">
      <Sidebar variant="admin" />
      <div className="flex flex-1 flex-col">
        <header className="flex h-16 items-center justify-between border-b bg-background px-6">
          <div className="flex items-center gap-4">
            <Link to="/dashboard">
              <Button variant="ghost" size="sm">
                <ArrowLeft className="h-4 w-4 mr-2" />
                Back to Dashboard
              </Button>
            </Link>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <User className="h-4 w-4" />
              <span>{user?.email}</span>
              <span className="rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                Admin
              </span>
            </div>
            <Button variant="ghost" size="sm" onClick={() => logout()}>
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </header>
        <main className="flex-1 overflow-auto p-6 bg-muted/30">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
