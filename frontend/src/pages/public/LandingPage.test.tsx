import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render } from '@/test/utils'
import { LandingPage } from './LandingPage'
import { useAuthStore } from '@/stores/authStore'
import { mockUser } from '@/test/mocks/handlers'

beforeEach(() => {
  useAuthStore.setState({
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
})

describe('LandingPage', () => {
  it('renders hero section', async () => {
    render(<LandingPage />)

    await waitFor(() => {
      // The hero shows one of the taglines
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })
  })

  it('shows features section', () => {
    render(<LandingPage />)

    expect(screen.getByText('No ops. No overhead. No nonsense.')).toBeInTheDocument()
    expect(screen.getByText('Blazing Fast')).toBeInTheDocument()
    expect(screen.getByText('Secure by Default')).toBeInTheDocument()
    expect(screen.getByText('$3/month. Forever.')).toBeInTheDocument()
  })

  it('shows toolkit section', () => {
    render(<LandingPage />)

    expect(screen.getByText('The toolkit')).toBeInTheDocument()
    expect(screen.getByText('RUS')).toBeInTheDocument()
    expect(screen.getByText('Rusty Links')).toBeInTheDocument()
  })

  it('shows Get Started button when not authenticated', () => {
    render(<LandingPage />)

    expect(screen.getAllByRole('button', { name: /get started/i }).length).toBeGreaterThan(0)
  })

  it('shows Go to Membership when authenticated', () => {
    useAuthStore.setState({ user: mockUser, isAuthenticated: true })

    render(<LandingPage />)

    expect(screen.getAllByRole('button', { name: /go to membership/i }).length).toBeGreaterThan(0)
  })

  it('shows pricing link', () => {
    render(<LandingPage />)

    expect(screen.getByRole('button', { name: /view pricing/i })).toBeInTheDocument()
  })

  it('shows CTA section', () => {
    render(<LandingPage />)

    expect(screen.getByText('Stop configuring. Start building.')).toBeInTheDocument()
  })
})
