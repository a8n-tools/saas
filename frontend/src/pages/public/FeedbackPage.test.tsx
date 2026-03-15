import { describe, it, expect } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { render } from '@/test/utils'
import { server } from '@/test/mocks/server'
import { FeedbackPage } from './FeedbackPage'

describe('FeedbackPage', () => {
  it('renders the feedback form', () => {
    render(<FeedbackPage />)
    expect(screen.getByText(/help shape what ships next/i)).toBeInTheDocument()
  })

  it('renders all form fields', () => {
    render(<FeedbackPage />)
    expect(screen.getByLabelText(/name/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/subject/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/message/i)).toBeInTheDocument()
  })

  it('renders tag buttons', () => {
    render(<FeedbackPage />)
    expect(screen.getByText('Bug')).toBeInTheDocument()
    expect(screen.getByText('Feature')).toBeInTheDocument()
    expect(screen.getByText('Flow')).toBeInTheDocument()
    expect(screen.getByText('Idea')).toBeInTheDocument()
  })

  it('honeypot field is present but hidden', () => {
    render(<FeedbackPage />)
    const honeypot = document.getElementById('website') as HTMLInputElement
    expect(honeypot).toBeInTheDocument()
    expect(honeypot.tabIndex).toBe(-1)
  })

  it('shows success state after valid submission', async () => {
    const user = userEvent.setup()
    render(<FeedbackPage />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/message/i), 'This is my feedback message.')
    await user.click(screen.getByRole('button', { name: /send feedback/i }))

    await waitFor(() => {
      expect(screen.getByText(/feedback sent/i)).toBeInTheDocument()
    })
  })

  it('shows error alert on API failure', async () => {
    server.use(
      http.post('*/feedback', () =>
        HttpResponse.json(
          { success: false, error: { code: 'SERVER_ERROR', message: 'Something went wrong' } },
          { status: 500 }
        )
      )
    )

    const user = userEvent.setup()
    render(<FeedbackPage />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/message/i), 'This is my feedback message.')
    await user.click(screen.getByRole('button', { name: /send feedback/i }))

    await waitFor(() => {
      expect(screen.getByText(/something went wrong/i)).toBeInTheDocument()
    })
  })

  it('resets form after successful submission', async () => {
    const user = userEvent.setup()
    render(<FeedbackPage />)

    const messageField = screen.getByLabelText(/message/i)
    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(messageField, 'This is my feedback.')
    await user.click(screen.getByRole('button', { name: /send feedback/i }))

    await waitFor(() => {
      expect(screen.getByText(/feedback sent/i)).toBeInTheDocument()
    })

    // After success the textarea is reset
    expect((messageField as HTMLTextAreaElement).value).toBe('')
  })
})
