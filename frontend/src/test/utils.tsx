import React, { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { TooltipProvider } from '@/components/ui/tooltip'
import { User } from '@/types'
import { mockUser, mockAdminUser, mockSuperAdminUser } from '@/test/mocks/handlers'

// Create a new QueryClient for each test
const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })

interface WrapperProps {
  children: React.ReactNode
}

function AllProviders({ children }: WrapperProps) {
  const queryClient = createTestQueryClient()

  return (
    // `TooltipProvider` mirrors `App.tsx`, which wraps the whole tree in one.
    // Any page rendering a `Tooltip` throws without it, so a wrapper missing it
    // fails tests for a reason the real app never has.
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <BrowserRouter>{children}</BrowserRouter>
      </TooltipProvider>
    </QueryClientProvider>
  )
}

const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) => render(ui, { wrapper: AllProviders, ...options })

// Re-export everything from testing-library
export * from '@testing-library/react'
export { customRender as render }

// Auth store setup helpers — use in beforeEach to avoid repeating the full setState boilerplate
export function setupAuthUser(user: User = mockUser) {
  useAuthStore.setState({
    user,
    isAuthenticated: true,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
}

export function setupAdminUser() {
  setupAuthUser(mockAdminUser)
}

// DEV-525: an admin that also holds the super-admin flag, for the four actions
// an ordinary admin may not take.
export function setupSuperAdminUser() {
  setupAuthUser(mockSuperAdminUser)
}

export function setupUnauthUser() {
  useAuthStore.setState({
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
}
