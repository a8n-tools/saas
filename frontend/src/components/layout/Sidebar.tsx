import { Link, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'
import { useAuthStore } from '@/stores/authStore'
import {
  LayoutDashboard,
  AppWindow,
  CreditCard,
  Settings,
  Users,
  FileText,
  Shield,
} from 'lucide-react'

interface SidebarItem {
  title: string
  href: string
  icon: React.ComponentType<{ className?: string }>
}

const dashboardItems: SidebarItem[] = [
  { title: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { title: 'Applications', href: '/applications', icon: AppWindow },
  { title: 'Subscription', href: '/subscription', icon: CreditCard },
  { title: 'Settings', href: '/settings', icon: Settings },
]

const adminItems: SidebarItem[] = [
  { title: 'Overview', href: '/admin', icon: LayoutDashboard },
  { title: 'Users', href: '/admin/users', icon: Users },
  { title: 'Subscriptions', href: '/admin/subscriptions', icon: CreditCard },
  { title: 'Applications', href: '/admin/applications', icon: AppWindow },
  { title: 'Audit Logs', href: '/admin/audit-logs', icon: FileText },
]

interface SidebarProps {
  variant?: 'dashboard' | 'admin'
}

export function Sidebar({ variant = 'dashboard' }: SidebarProps) {
  const location = useLocation()
  const { user } = useAuthStore()
  const items = variant === 'admin' ? adminItems : dashboardItems
  const isAdmin = user?.role === 'admin'

  return (
    <aside className="hidden md:flex w-64 flex-col border-r bg-background">
      <div className="flex h-16 items-center border-b px-6">
        <Link to="/" className="flex items-center space-x-2">
          <span className="text-xl font-bold text-primary">a8n</span>
          <span className="text-xl font-light">.tools</span>
        </Link>
        {variant === 'admin' && (
          <span className="ml-2 rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
            Admin
          </span>
        )}
      </div>
      <nav className="flex-1 space-y-1 p-4">
        {items.map((item) => {
          const isActive = location.pathname === item.href
          return (
            <Link
              key={item.href}
              to={item.href}
              className={cn(
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors',
                isActive
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
              )}
            >
              <item.icon className="h-4 w-4" />
              {item.title}
            </Link>
          )
        })}
        {variant === 'dashboard' && isAdmin && (
          <>
            <div className="my-4 border-t" />
            <Link
              to="/admin"
              className={cn(
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors',
                'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
              )}
            >
              <Shield className="h-4 w-4" />
              Admin Panel
            </Link>
          </>
        )}
        {variant === 'admin' && (
          <>
            <div className="my-4 border-t" />
            <Link
              to="/dashboard"
              className={cn(
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors',
                'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
              )}
            >
              <LayoutDashboard className="h-4 w-4" />
              User Dashboard
            </Link>
          </>
        )}
      </nav>
    </aside>
  )
}
