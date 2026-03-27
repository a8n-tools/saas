import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render, setupAuthUser } from '@/test/utils'
import { CheckoutSuccessPage } from './CheckoutSuccessPage'

beforeEach(() => {
  setupAuthUser()
})

describe('CheckoutSuccessPage', () => {
  it('renders success message', () => {
    render(<CheckoutSuccessPage />)

    expect(screen.getByText(/welcome aboard/i)).toBeInTheDocument()
    expect(screen.getByText(/membership is now active/i)).toBeInTheDocument()
  })

  it('shows membership tier info for logged in user', async () => {
    render(<CheckoutSuccessPage />)

    await waitFor(() => {
      expect(screen.getByText(/standard plan/i)).toBeInTheDocument()
    })
  })

  it('shows navigation buttons', () => {
    render(<CheckoutSuccessPage />)

    expect(screen.getByRole('button', { name: /browse applications/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /view membership details/i })).toBeInTheDocument()
  })

  it('shows countdown timer', async () => {
    render(<CheckoutSuccessPage />)

    await waitFor(() => {
      expect(screen.getByText(/redirecting to applications in/i)).toBeInTheDocument()
    })
  })
})
