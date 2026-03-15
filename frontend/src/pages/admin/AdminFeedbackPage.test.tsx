import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { render, setupAdminUser } from '@/test/utils'
import { server } from '@/test/mocks/server'
import { mockFeedbackDetail } from '@/test/mocks/handlers'
import { AdminFeedbackPage } from './AdminFeedbackPage'

beforeEach(() => {
  setupAdminUser()
})

describe('AdminFeedbackPage', () => {
  it('renders page heading', () => {
    render(<AdminFeedbackPage />)
    expect(screen.getByText('Feedback')).toBeInTheDocument()
  })

  it('shows status filter buttons', () => {
    render(<AdminFeedbackPage />)
    expect(screen.getByRole('button', { name: 'All' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'New' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Reviewed' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Responded' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Closed' })).toBeInTheDocument()
  })

  it('renders feedback list from API', async () => {
    render(<AdminFeedbackPage />)
    await waitFor(() => {
      expect(screen.getByText('Login issue')).toBeInTheDocument()
    })
    expect(screen.getByText('I cannot log in with my credentials.')).toBeInTheDocument()
  })

  it('shows capitalized status badge in list', async () => {
    render(<AdminFeedbackPage />)
    await waitFor(() => {
      expect(screen.getByText('New')).toBeInTheDocument()
    })
  })

  it('shows empty state when no feedback', async () => {
    server.use(
      http.get('*/admin/feedback', () =>
        HttpResponse.json({ success: true, data: { items: [], total: 0, page: 1, total_pages: 1 } })
      )
    )
    render(<AdminFeedbackPage />)
    await waitFor(() => {
      expect(screen.getByText(/no feedback found/i)).toBeInTheDocument()
    })
  })

  it('opens detail dialog on item click', async () => {
    const user = userEvent.setup()
    render(<AdminFeedbackPage />)

    await waitFor(() => {
      expect(screen.getByText('Login issue')).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /open/i }))

    await waitFor(() => {
      expect(screen.getByText('Feedback detail')).toBeInTheDocument()
    })
  })

  it('shows capitalized status badge in detail dialog', async () => {
    const user = userEvent.setup()
    render(<AdminFeedbackPage />)

    await waitFor(() => screen.getByRole('button', { name: /open/i }))
    await user.click(screen.getByRole('button', { name: /open/i }))

    await waitFor(() => {
      expect(screen.getAllByText('New').length).toBeGreaterThan(0)
    })
  })

  it('does not show delete button for non-closed feedback', async () => {
    const user = userEvent.setup()
    render(<AdminFeedbackPage />)

    await waitFor(() => screen.getByRole('button', { name: /open/i }))
    await user.click(screen.getByRole('button', { name: /open/i }))

    await waitFor(() => {
      expect(screen.getByText('Feedback detail')).toBeInTheDocument()
    })

    expect(screen.queryByRole('button', { name: /^delete$/i })).not.toBeInTheDocument()
  })

  it('shows delete button for closed feedback', async () => {
    server.use(
      http.get('*/admin/feedback/:id', () =>
        HttpResponse.json({ success: true, data: { ...mockFeedbackDetail, status: 'closed' } })
      )
    )

    const user = userEvent.setup()
    render(<AdminFeedbackPage />)

    await waitFor(() => screen.getByRole('button', { name: /open/i }))
    await user.click(screen.getByRole('button', { name: /open/i }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /^delete$/i })).toBeInTheDocument()
    })
  })

  it('shows delete confirmation dialog', async () => {
    server.use(
      http.get('*/admin/feedback/:id', () =>
        HttpResponse.json({ success: true, data: { ...mockFeedbackDetail, status: 'closed' } })
      )
    )

    const user = userEvent.setup()
    render(<AdminFeedbackPage />)

    await waitFor(() => screen.getByRole('button', { name: /open/i }))
    await user.click(screen.getByRole('button', { name: /open/i }))

    await waitFor(() => screen.getByRole('button', { name: /^delete$/i }))
    await user.click(screen.getByRole('button', { name: /^delete$/i }))

    await waitFor(() => {
      expect(screen.getByText('Permanently delete this feedback?')).toBeInTheDocument()
      expect(screen.getByText('This action cannot be undone.')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /delete permanently/i })).toBeInTheDocument()
    })
  })

  it('closes detail dialog after confirmed delete', async () => {
    server.use(
      http.get('*/admin/feedback/:id', () =>
        HttpResponse.json({ success: true, data: { ...mockFeedbackDetail, status: 'closed' } })
      )
    )

    const user = userEvent.setup()
    render(<AdminFeedbackPage />)

    await waitFor(() => screen.getByRole('button', { name: /open/i }))
    await user.click(screen.getByRole('button', { name: /open/i }))

    await waitFor(() => screen.getByRole('button', { name: /^delete$/i }))
    await user.click(screen.getByRole('button', { name: /^delete$/i }))

    await waitFor(() => screen.getByRole('button', { name: /delete permanently/i }))
    await user.click(screen.getByRole('button', { name: /delete permanently/i }))

    await waitFor(() => {
      expect(screen.queryByText('Feedback detail')).not.toBeInTheDocument()
    })
  })

  it('shows mark closed button for non-closed feedback', async () => {
    const user = userEvent.setup()
    render(<AdminFeedbackPage />)

    await waitFor(() => screen.getByRole('button', { name: /open/i }))
    await user.click(screen.getByRole('button', { name: /open/i }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /mark closed/i })).toBeInTheDocument()
    })
  })
})
