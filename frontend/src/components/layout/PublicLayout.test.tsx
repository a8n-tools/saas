import { describe, it, expect } from 'vitest'
import { render, screen } from '@/test/utils'
import { PublicLayout } from './PublicLayout'

describe('PublicLayout', () => {
  it('renders header and footer', () => {
    render(<PublicLayout />)
    // Header contains logo/nav, Footer contains copyright info
    // The Footer renders the site name/logo
    expect(screen.getByText(/Terms/i)).toBeInTheDocument()
    expect(screen.getByText(/Privacy/i)).toBeInTheDocument()
  })
})
