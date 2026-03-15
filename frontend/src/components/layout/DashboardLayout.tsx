import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { useAuthStore } from '@/stores/authStore'
import { useThemeStore } from '@/stores/themeStore'
import { Button } from '@/components/ui/button'
import { LogOut, Moon, Sun, User } from 'lucide-react'

export function DashboardLayout() {
  const { user, logout } = useAuthStore()
  const { theme, setTheme } = useThemeStore()

  const toggleTheme = () => {
    setTheme(theme === 'dark' ? 'light' : 'dark')
  }

  return (
    <div className="flex min-h-screen">
      <Sidebar variant="dashboard" />
      <div className="flex flex-1 flex-col">
        <header className="flex h-16 items-center justify-between border-b border-border/50 bg-background/80 backdrop-blur-sm px-6">
          <h1 className="text-lg font-semibold">Dashboard</h1>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <User className="h-4 w-4" />
              <span>{user?.email}</span>
            </div>
            <Button variant="ghost" size="icon" onClick={toggleTheme} aria-label="Toggle theme">
              <Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
              <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
            </Button>
            <Button variant="ghost" size="sm" onClick={() => logout()}>
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </header>
        <main className="relative flex-1 overflow-auto p-6">
          {/* Subtle background gradient */}
          <div className="pointer-events-none absolute inset-0 bg-gradient-to-br from-indigo-500/[0.02] via-transparent to-teal-500/[0.02]" />
          <div className="relative">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  )
}
