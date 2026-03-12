import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import { render } from '@/test/utils'
import { MembershipRequiredPage } from './MembershipRequiredPage'

describe('MembershipRequiredPage', () => {
  it('renders membership required content', () => {
    render(<MembershipRequiredPage />)

    expect(screen.getByText('Membership Required')).toBeInTheDocument()
    expect(screen.getByText(/active membership to access/i)).toBeInTheDocument()
    expect(screen.getByText(/\$3\/month/i)).toBeInTheDocument()
  })

  it('shows subscribe and back to dashboard buttons', () => {
    render(<MembershipRequiredPage />)

    expect(screen.getByRole('button', { name: /subscribe now/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /back to dashboard/i })).toBeInTheDocument()
  })
})
