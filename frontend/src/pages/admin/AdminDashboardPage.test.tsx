import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render } from '@/test/utils'
import { AdminDashboardPage } from './AdminDashboardPage'
import { useAuthStore } from '@/stores/authStore'
import { mockAdminUser } from '@/test/mocks/handlers'

beforeEach(() => {
  useAuthStore.setState({
    user: mockAdminUser,
    isAuthenticated: true,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
})

describe('AdminDashboardPage', () => {
  it('renders admin dashboard heading', () => {
    render(<AdminDashboardPage />)

    expect(screen.getByText('Admin Dashboard')).toBeInTheDocument()
    expect(screen.getByText('Overview of your platform.')).toBeInTheDocument()
  })

  it('shows stats cards', async () => {
    render(<AdminDashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('Total Users')).toBeInTheDocument()
      expect(screen.getByText('Active Memberships')).toBeInTheDocument()
      expect(screen.getByText('Active Apps')).toBeInTheDocument()
      expect(screen.getByText('Past Due')).toBeInTheDocument()
    })
  })

  it('shows stat values from API', async () => {
    render(<AdminDashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('100')).toBeInTheDocument() // total_users
      expect(screen.getByText('75')).toBeInTheDocument()  // active_members
    })
  })

  it('shows recent activity section', async () => {
    render(<AdminDashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('Recent Activity')).toBeInTheDocument()
    })
  })

  it('shows recent audit log entries', async () => {
    render(<AdminDashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('User Login')).toBeInTheDocument()
    })
  })

  it('shows system health section', async () => {
    render(<AdminDashboardPage />)

    await waitFor(() => {
      expect(screen.getByText('System Health')).toBeInTheDocument()
      expect(screen.getByText('Healthy')).toBeInTheDocument()
    })
  })
})
