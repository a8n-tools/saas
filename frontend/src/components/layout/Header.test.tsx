import { describe, it, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupUnauthUser } from '@/test/utils'
import { Header } from './Header'
import { useAuthStore } from '@/stores/authStore'
import { useThemeStore } from '@/stores/themeStore'
import { mockUser, mockAdminUser } from '@/test/mocks/handlers'

beforeEach(() => {
  setupUnauthUser()
})

describe('Header', () => {
  it('renders logo', () => {
    render(<Header />)

    expect(screen.getByText('a8n')).toBeInTheDocument()
    expect(screen.getByText('.tools')).toBeInTheDocument()
  })

  it('shows login and register buttons when not authenticated', () => {
    render(<Header />)

    expect(screen.getByRole('button', { name: /login/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /get started/i })).toBeInTheDocument()
  })

  it('shows dashboard and logout buttons when authenticated', () => {
    useAuthStore.setState({ user: mockUser, isAuthenticated: true })

    render(<Header />)

    expect(screen.getByRole('button', { name: /dashboard/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /logout/i })).toBeInTheDocument()
  })

  it('does not show admin button for regular user', () => {
    useAuthStore.setState({ user: mockUser, isAuthenticated: true })

    render(<Header />)

    expect(screen.queryByRole('button', { name: /admin/i })).not.toBeInTheDocument()
  })

  it('shows admin button for admin users', () => {
    useAuthStore.setState({ user: mockAdminUser, isAuthenticated: true })

    render(<Header />)

    expect(screen.getByRole('button', { name: /admin/i })).toBeInTheDocument()
  })

  it('shows pricing link in nav', () => {
    render(<Header />)

    expect(screen.getByText('Pricing')).toBeInTheDocument()
  })

  it('shows theme toggle button', () => {
    render(<Header />)

    expect(screen.getByRole('button', { name: /toggle theme/i })).toBeInTheDocument()
  })

  it('toggles theme on click', async () => {
    const user = userEvent.setup()
    render(<Header />)

    const initialTheme = useThemeStore.getState().theme
    await user.click(screen.getByRole('button', { name: /toggle theme/i }))

    const newTheme = useThemeStore.getState().theme
    expect(newTheme).not.toBe(initialTheme)
  })

  it('calls logout when logout button clicked', async () => {
    const user = userEvent.setup()
    useAuthStore.setState({ user: mockUser, isAuthenticated: true })

    render(<Header />)

    await user.click(screen.getByRole('button', { name: /logout/i }))

    // After logout, the store should clear
    expect(useAuthStore.getState().isAuthenticated).toBe(false)
  })
})
