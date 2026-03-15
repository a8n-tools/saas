import { describe, it, expect } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render } from '@/test/utils'
import { Footer } from './Footer'

describe('Footer', () => {
  it('renders logo', () => {
    render(<Footer />)

    expect(screen.getByText('a8n')).toBeInTheDocument()
    expect(screen.getAllByText('.tools').length).toBeGreaterThan(0)
  })

  it('renders product links', async () => {
    render(<Footer />)

    await waitFor(() => {
      expect(screen.getByText('Pricing')).toBeInTheDocument()
      expect(screen.getByText('RUS')).toBeInTheDocument()
      expect(screen.getByText('Rusty Links')).toBeInTheDocument()
    })
  })

  it('renders account links', () => {
    render(<Footer />)

    expect(screen.getByText('Login')).toBeInTheDocument()
    expect(screen.getByText('Register')).toBeInTheDocument()
  })

  it('renders legal links', () => {
    render(<Footer />)

    expect(screen.getByText('Terms of Service')).toBeInTheDocument()
    expect(screen.getByText('Privacy Policy')).toBeInTheDocument()
  })

  it('shows tagline', () => {
    render(<Footer />)

    expect(screen.getByText('Ship more. Manage less.')).toBeInTheDocument()
  })

  it('shows copyright text', () => {
    render(<Footer />)

    expect(screen.getByText(/all rights reserved/i)).toBeInTheDocument()
  })
})
