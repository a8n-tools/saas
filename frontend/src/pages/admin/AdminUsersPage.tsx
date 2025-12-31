import { useState } from 'react'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Search, MoreVertical, User } from 'lucide-react'

export function AdminUsersPage() {
  const [searchQuery, setSearchQuery] = useState('')

  // TODO: Fetch actual users from API
  const users = [
    {
      id: '1',
      email: 'user1@example.com',
      role: 'subscriber',
      subscription_status: 'active',
      created_at: '2024-12-01T00:00:00Z',
    },
    {
      id: '2',
      email: 'user2@example.com',
      role: 'subscriber',
      subscription_status: 'past_due',
      created_at: '2024-12-15T00:00:00Z',
    },
    {
      id: '3',
      email: 'admin@example.com',
      role: 'admin',
      subscription_status: 'active',
      created_at: '2024-11-01T00:00:00Z',
    },
  ]

  const filteredUsers = users.filter((user) =>
    user.email.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Users</h1>
          <p className="mt-2 text-muted-foreground">
            Manage user accounts and subscriptions.
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-4">
            <div className="relative flex-1 max-w-sm">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search users..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9"
              />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {filteredUsers.map((user) => (
              <div
                key={user.id}
                className="flex items-center justify-between py-4 border-b last:border-0"
              >
                <div className="flex items-center gap-4">
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-muted">
                    <User className="h-5 w-5 text-muted-foreground" />
                  </div>
                  <div>
                    <p className="font-medium">{user.email}</p>
                    <p className="text-sm text-muted-foreground">
                      Joined {new Date(user.created_at).toLocaleDateString()}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  <Badge
                    variant={user.role === 'admin' ? 'default' : 'secondary'}
                  >
                    {user.role}
                  </Badge>
                  <SubscriptionBadge status={user.subscription_status} />
                  <Button variant="ghost" size="icon">
                    <MoreVertical className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function SubscriptionBadge({ status }: { status: string }) {
  switch (status) {
    case 'active':
      return <Badge variant="success">Active</Badge>
    case 'past_due':
      return <Badge variant="warning">Past Due</Badge>
    case 'canceled':
      return <Badge variant="destructive">Canceled</Badge>
    default:
      return <Badge variant="outline">None</Badge>
  }
}
