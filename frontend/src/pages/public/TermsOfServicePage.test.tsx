import { describe, it, expect } from 'vitest'
import { render, screen } from '@/test/utils'
import { TermsOfServicePage } from './TermsOfServicePage'

describe('TermsOfServicePage', () => {
  it('renders the page heading', () => {
    render(<TermsOfServicePage />)
    expect(screen.getByText('Terms of Service')).toBeInTheDocument()
  })

  it('shows last updated date', () => {
    render(<TermsOfServicePage />)
    expect(screen.getByText(/Last updated/)).toBeInTheDocument()
  })

  it('renders key sections', () => {
    render(<TermsOfServicePage />)
    expect(screen.getByText('1. Acceptance of Terms')).toBeInTheDocument()
    expect(screen.getByText('3. Membership and Payment')).toBeInTheDocument()
    expect(screen.getByText('6. Acceptable Use')).toBeInTheDocument()
    expect(screen.getByText('11. Termination')).toBeInTheDocument()
  })

  it('renders support contact email link', () => {
    render(<TermsOfServicePage />)
    const emailLink = screen.getByRole('link', { name: /support@/ })
    expect(emailLink).toBeInTheDocument()
  })
})
