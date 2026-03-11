import { describe, it, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import { render } from '@/test/utils'
import { PricingPage } from './PricingPage'
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

describe('PricingPage', () => {
  it('renders pricing information', () => {
    render(<PricingPage />)

    expect(screen.getByText('Simple, Transparent Pricing')).toBeInTheDocument()
    expect(screen.getByText('Personal')).toBeInTheDocument()
    expect(screen.getByText('$3')).toBeInTheDocument()
  })

  it('shows personal plan features', () => {
    render(<PricingPage />)

    expect(screen.getByText('Access to all current applications')).toBeInTheDocument()
    expect(screen.getByText('Price locked for life')).toBeInTheDocument()
    expect(screen.getByText('Cancel anytime')).toBeInTheDocument()
  })

  it('shows FAQ section', () => {
    render(<PricingPage />)

    expect(screen.getByText('Frequently Asked Questions')).toBeInTheDocument()
    // "price locked for life" appears in both features and FAQ sections
    expect(screen.getAllByText(/price locked for life/i).length).toBeGreaterThan(0)
  })

  it('shows "Get Started" button when not authenticated', () => {
    render(<PricingPage />)

    expect(screen.getByRole('button', { name: /get started/i })).toBeInTheDocument()
  })

  it('shows "Go to Membership" button when authenticated', () => {
    useAuthStore.setState({ user: mockUser, isAuthenticated: true })

    render(<PricingPage />)

    expect(screen.getByRole('button', { name: /go to membership/i })).toBeInTheDocument()
  })
})
