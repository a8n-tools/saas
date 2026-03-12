import { describe, it, expect, beforeEach, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render, setupAuthUser } from '@/test/utils'
import { MembershipPage } from './MembershipPage'
import { useMembershipStore } from '@/stores/membershipStore'
import { mockMembership } from '@/test/mocks/handlers'

beforeEach(() => {
  setupAuthUser()
  // Override fetchMembership to prevent auto-fetching on mount
  // TODO: as never cast needed because Zustand setState doesn't accept partial function overrides — fix store type
  useMembershipStore.setState({
    membership: null,
    isLoading: false,
    error: null,
    fetchMembership: vi.fn().mockResolvedValue(undefined),
  } as never)
})

describe('MembershipPage', () => {
  it('renders page heading', () => {
    render(<MembershipPage />)

    expect(screen.getByText('Membership')).toBeInTheDocument()
    expect(screen.getByText('Manage your membership and billing.')).toBeInTheDocument()
  })

  it('shows loading spinner initially', () => {
    useMembershipStore.setState({ isLoading: true })

    render(<MembershipPage />)

    // TODO: query by role="status" or aria-label instead of CSS class
    expect(document.querySelector('.animate-spin')).toBeInTheDocument()
  })

  it('shows active membership details', async () => {
    useMembershipStore.setState({
      membership: mockMembership,
      isLoading: false,
    })

    render(<MembershipPage />)

    expect(screen.getByText('Current Plan')).toBeInTheDocument()
    expect(screen.getByText('Cancel Membership')).toBeInTheDocument()
    expect(screen.getByText('Cancel Now')).toBeInTheDocument()
  })

  it('shows no membership state with tier selection', async () => {
    useMembershipStore.setState({ membership: null, isLoading: false })

    render(<MembershipPage />)

    await waitFor(() => {
      expect(screen.getByText('No Active Membership')).toBeInTheDocument()
    })

    expect(screen.getByText('Subscribe to Personal')).toBeInTheDocument()
  })

  it('shows tier selector buttons', async () => {
    useMembershipStore.setState({ membership: null, isLoading: false })

    render(<MembershipPage />)

    await waitFor(() => {
      expect(screen.getByText('Personal')).toBeInTheDocument()
      expect(screen.getByText('Business')).toBeInTheDocument()
    })
  })

  it('shows payment history section', () => {
    render(<MembershipPage />)

    expect(screen.getByText('Payment History')).toBeInTheDocument()
  })

  it('shows error alert when there is an error', () => {
    useMembershipStore.setState({ error: 'Failed to load membership', isLoading: false })

    render(<MembershipPage />)

    expect(screen.getByText('Failed to load membership')).toBeInTheDocument()
  })
})
