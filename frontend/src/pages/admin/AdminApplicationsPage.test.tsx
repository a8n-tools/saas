import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAdminUser } from '@/test/utils'
import { AdminApplicationsPage } from './AdminApplicationsPage'

beforeEach(() => {
  setupAdminUser()
})

describe('AdminApplicationsPage', () => {
  it('renders applications page heading', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByText('Applications')).toBeInTheDocument()
      expect(screen.getByText('Manage platform applications.')).toBeInTheDocument()
    })
  })

  it('shows application cards after loading', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByText('RUS')).toBeInTheDocument()
    })
  })

  it('shows active/inactive badges', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      // Multiple "Active" elements may exist (badge + toggle label)
      expect(screen.getAllByText('Active').length).toBeGreaterThan(0)
    })
  })

  it('shows edit button for each application', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument()
    })
  })

  it('opens edit dialog when edit button clicked', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /edit/i }))

    expect(screen.getByText(/edit rus/i)).toBeInTheDocument()
  })

  it('edit dialog has required fields', async () => {
    const user = userEvent.setup()
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /edit/i })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /edit/i }))

    expect(screen.getByLabelText('Display Name *')).toBeInTheDocument()
    expect(screen.getByLabelText('Container Name *')).toBeInTheDocument()
  })

  it('shows toggle switches for active and maintenance mode', async () => {
    render(<AdminApplicationsPage />)

    await waitFor(() => {
      // Multiple "Active" elements may exist (badge + toggle label)
      expect(screen.getAllByText('Active').length).toBeGreaterThan(0)
      expect(screen.getByText('Maintenance Mode')).toBeInTheDocument()
    })
  })
})
