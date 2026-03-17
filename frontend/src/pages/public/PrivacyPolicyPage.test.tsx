import { describe, it, expect } from 'vitest'
import { render, screen } from '@/test/utils'
import { PrivacyPolicyPage } from './PrivacyPolicyPage'

describe('PrivacyPolicyPage', () => {
  it('renders the page heading', () => {
    render(<PrivacyPolicyPage />)
    expect(screen.getByText('Privacy Policy')).toBeInTheDocument()
  })

  it('shows last updated date', () => {
    render(<PrivacyPolicyPage />)
    expect(screen.getByText(/Last updated/)).toBeInTheDocument()
  })

  it('renders all major sections', () => {
    render(<PrivacyPolicyPage />)
    expect(screen.getByText('1. Introduction')).toBeInTheDocument()
    expect(screen.getByText('2. Information We Collect')).toBeInTheDocument()
    expect(screen.getByText('5. Data Security')).toBeInTheDocument()
    expect(screen.getByText('7. Your Rights')).toBeInTheDocument()
    expect(screen.getByText('13. Contact Us')).toBeInTheDocument()
  })

  it('renders privacy contact email link', () => {
    render(<PrivacyPolicyPage />)
    const emailLinks = screen.getAllByRole('link', { name: /privacy@/ })
    expect(emailLinks.length).toBeGreaterThan(0)
  })
})
