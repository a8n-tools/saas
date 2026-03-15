import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render, setupAdminUser } from '@/test/utils'
import { AdminMembershipsPage } from './AdminMembershipsPage'

beforeEach(() => {
  setupAdminUser()
})

describe('AdminMembershipsPage', () => {
  it('renders memberships page heading', () => {
    render(<AdminMembershipsPage />)

    expect(screen.getByText('Memberships')).toBeInTheDocument()
    expect(screen.getByText('View and manage all memberships.')).toBeInTheDocument()
  })

  it('shows stats cards', async () => {
    render(<AdminMembershipsPage />)

    await waitFor(() => {
      expect(screen.getByText('Active')).toBeInTheDocument()
      expect(screen.getByText('Past Due')).toBeInTheDocument()
      expect(screen.getByText('Grace Period')).toBeInTheDocument()
    })
  })

  it('shows stat values from API', async () => {
    render(<AdminMembershipsPage />)

    await waitFor(() => {
      expect(screen.getByText('75')).toBeInTheDocument() // active_members
      expect(screen.getByText('5')).toBeInTheDocument()  // past_due_members
    })
  })

  it('shows memberships list after loading', async () => {
    render(<AdminMembershipsPage />)

    await waitFor(() => {
      expect(screen.getByText('test@example.com')).toBeInTheDocument()
    })
  })

  it('shows status filter dropdown', () => {
    render(<AdminMembershipsPage />)

    expect(screen.getByText('All Status')).toBeInTheDocument()
  })

  it('shows all memberships heading', () => {
    render(<AdminMembershipsPage />)

    expect(screen.getByText('All Memberships')).toBeInTheDocument()
  })
})
