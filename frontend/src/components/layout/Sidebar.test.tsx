import { describe, it, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import { render, setupAuthUser } from '@/test/utils'
import { Sidebar } from './Sidebar'
import { useAuthStore } from '@/stores/authStore'
import { mockAdminUser } from '@/test/mocks/handlers'

beforeEach(() => {
  setupAuthUser()
})

describe('Sidebar', () => {
  describe('dashboard variant', () => {
    it('renders dashboard navigation items', () => {
      render(<Sidebar variant="dashboard" />)

      expect(screen.getByText('Dashboard')).toBeInTheDocument()
      expect(screen.getByText('Applications')).toBeInTheDocument()
      expect(screen.getByText('Membership')).toBeInTheDocument()
      expect(screen.getByText('Settings')).toBeInTheDocument()
    })

    it('shows logo', () => {
      render(<Sidebar variant="dashboard" />)

      expect(screen.getByText('a8n')).toBeInTheDocument()
    })

    it('does not show Admin label', () => {
      render(<Sidebar variant="dashboard" />)

      expect(screen.queryByText('Admin')).not.toBeInTheDocument()
    })

    it('shows admin panel link for admin users', () => {
      useAuthStore.setState({ user: mockAdminUser, isAuthenticated: true })

      render(<Sidebar variant="dashboard" />)

      expect(screen.getByText('Admin Panel')).toBeInTheDocument()
    })

    it('does not show admin panel link for regular users', () => {
      render(<Sidebar variant="dashboard" />)

      expect(screen.queryByText('Admin Panel')).not.toBeInTheDocument()
    })
  })

  describe('admin variant', () => {
    it('renders admin navigation items', () => {
      render(<Sidebar variant="admin" />)

      expect(screen.getByText('Overview')).toBeInTheDocument()
      expect(screen.getByText('Users')).toBeInTheDocument()
      expect(screen.getByText('Memberships')).toBeInTheDocument()
      expect(screen.getByText('Audit Logs')).toBeInTheDocument()
    })

    it('shows Admin label', () => {
      render(<Sidebar variant="admin" />)

      expect(screen.getByText('Admin')).toBeInTheDocument()
    })

    it('shows user dashboard link', () => {
      render(<Sidebar variant="admin" />)

      expect(screen.getByText('User Dashboard')).toBeInTheDocument()
    })
  })
})
