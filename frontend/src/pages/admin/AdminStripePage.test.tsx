import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAdminUser } from '@/test/utils'
import { AdminStripePage } from './AdminStripePage'

beforeEach(() => {
  setupAdminUser()
})

describe('AdminStripePage', () => {
  it('renders the page title and description', async () => {
    render(<AdminStripePage />)

    await waitFor(() => {
      expect(screen.getByText('Stripe')).toBeInTheDocument()
    })
    expect(screen.getByText(/manage your stripe payment integration/i)).toBeInTheDocument()
  })

  it('shows current config source badge', async () => {
    render(<AdminStripePage />)

    await waitFor(() => {
      expect(screen.getByText('Database')).toBeInTheDocument()
    })
  })

  it('shows masked current values', async () => {
    render(<AdminStripePage />)

    await waitFor(() => {
      expect(screen.getByText('***1234')).toBeInTheDocument()
    })
    expect(screen.getByText('***5678')).toBeInTheDocument()
    expect(screen.getByText('price_personal_123')).toBeInTheDocument()
    expect(screen.getByText('price_business_456')).toBeInTheDocument()
  })

  it('renders form fields', async () => {
    render(<AdminStripePage />)

    await waitFor(() => {
      expect(screen.getByLabelText('Secret Key')).toBeInTheDocument()
    })
    expect(screen.getByLabelText('Webhook Secret')).toBeInTheDocument()
    expect(screen.getByLabelText('Personal Plan Price ID')).toBeInTheDocument()
    expect(screen.getByLabelText('Business Plan Price ID')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /save changes/i })).toBeInTheDocument()
  })

  it('shows success message after saving', async () => {
    const user = userEvent.setup()
    render(<AdminStripePage />)

    await waitFor(() => {
      expect(screen.getByLabelText('Secret Key')).toBeInTheDocument()
    })

    await user.type(screen.getByLabelText('Secret Key'), 'sk_live_newkey1')
    await user.click(screen.getByRole('button', { name: /save changes/i }))

    await waitFor(() => {
      expect(screen.getByText(/stripe configuration updated successfully/i)).toBeInTheDocument()
    })
  })
})
