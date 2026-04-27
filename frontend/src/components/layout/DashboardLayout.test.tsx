import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@/test/utils'
import { setupAuthUser } from '@/test/utils'
import { DashboardLayout } from './DashboardLayout'

describe('DashboardLayout', () => {
  beforeEach(() => {
    setupAuthUser()
  })

  it('renders the Dashboard heading', () => {
    render(<DashboardLayout />)
    // "Dashboard" appears in both header h1 and sidebar nav
    const headings = screen.getAllByText('Dashboard')
    expect(headings.length).toBeGreaterThanOrEqual(1)
  })

  it('displays user email', () => {
    render(<DashboardLayout />)
    expect(screen.getByText('test@example.com')).toBeInTheDocument()
  })

  it('renders theme toggle button', () => {
    render(<DashboardLayout />)
    expect(screen.getByLabelText('Toggle theme')).toBeInTheDocument()
  })

  it('renders high contrast toggle button', () => {
    render(<DashboardLayout />)
    expect(screen.getByLabelText('Toggle high contrast')).toBeInTheDocument()
  })

  it('renders sidebar with navigation', () => {
    render(<DashboardLayout />)
    // Sidebar dashboard variant has a "Dashboard" nav link
    const navLinks = screen.getAllByText('Dashboard')
    expect(navLinks.length).toBeGreaterThanOrEqual(1)
  })
})
