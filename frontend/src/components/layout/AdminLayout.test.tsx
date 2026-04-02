import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@/test/utils'
import { setupAdminUser } from '@/test/utils'
import { AdminLayout } from './AdminLayout'

describe('AdminLayout', () => {
  beforeEach(() => {
    setupAdminUser()
  })

  it('displays admin user email', () => {
    render(<AdminLayout />)
    expect(screen.getByText('admin@example.com')).toBeInTheDocument()
  })

  it('shows Admin badge', () => {
    render(<AdminLayout />)
    const badges = screen.getAllByText('Admin')
    expect(badges.length).toBeGreaterThanOrEqual(1)
  })

  it('renders Back to Dashboard link', () => {
    render(<AdminLayout />)
    expect(screen.getByText('Back to Dashboard')).toBeInTheDocument()
  })

  it('renders theme toggle button', () => {
    render(<AdminLayout />)
    expect(screen.getByLabelText('Toggle theme')).toBeInTheDocument()
  })

  it('renders high contrast toggle button', () => {
    render(<AdminLayout />)
    expect(screen.getByLabelText('Toggle high contrast')).toBeInTheDocument()
  })
})
