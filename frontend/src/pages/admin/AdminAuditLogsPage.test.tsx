import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '@/test/utils'
import { AdminAuditLogsPage } from './AdminAuditLogsPage'
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

describe('AdminAuditLogsPage', () => {
  it('renders audit logs heading', () => {
    render(<AdminAuditLogsPage />)

    expect(screen.getByText('Audit Logs')).toBeInTheDocument()
    expect(screen.getByText('View security events and user activity.')).toBeInTheDocument()
  })

  it('shows recent activity section', () => {
    render(<AdminAuditLogsPage />)

    expect(screen.getByText('Recent Activity')).toBeInTheDocument()
  })

  it('shows admin-only toggle', () => {
    render(<AdminAuditLogsPage />)

    expect(screen.getByText('Admin actions only')).toBeInTheDocument()
  })

  it('shows audit log entries after loading', async () => {
    render(<AdminAuditLogsPage />)

    await waitFor(() => {
      expect(screen.getByText('User Login')).toBeInTheDocument()
    })
  })

  it('shows actor email in log entry', async () => {
    render(<AdminAuditLogsPage />)

    await waitFor(() => {
      expect(screen.getByText(/test@example.com/)).toBeInTheDocument()
    })
  })

  it('toggles admin-only filter', async () => {
    const user = userEvent.setup()
    render(<AdminAuditLogsPage />)

    const toggle = screen.getByRole('switch')
    await user.click(toggle)

    // The switch should be checked now
    expect(toggle).toBeChecked()
  })
})
