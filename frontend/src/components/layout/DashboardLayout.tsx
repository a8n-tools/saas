import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { useAuthStore } from '@/stores/authStore'
import { Button } from '@/components/ui/button'
import { LogOut, User } from 'lucide-react'

export function DashboardLayout() {
  const { user, logout } = useAuthStore()

  return (
    <div className="flex min-h-screen">
      <Sidebar variant="dashboard" />
      <div className="flex flex-1 flex-col">
        <header className="flex h-16 items-center justify-between border-b bg-background px-6">
          <h1 className="text-lg font-semibold">Dashboard</h1>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <User className="h-4 w-4" />
              <span>{user?.email}</span>
            </div>
            <Button variant="ghost" size="sm" onClick={() => logout()}>
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </header>
        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
