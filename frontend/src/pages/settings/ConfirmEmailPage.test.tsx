import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { render as rtlRender } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ConfirmEmailPage } from './ConfirmEmailPage'
import { setupAuthUser } from '@/test/utils'

function renderWithSearch(search = '') {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return rtlRender(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[`/settings/confirm-email${search}`]}>
        <Routes>
          <Route path="/settings/confirm-email" element={<ConfirmEmailPage />} />
          <Route path="/login" element={<div>Login Page</div>} />
          <Route path="/settings" element={<div>Settings Page</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

beforeEach(() => {
  setupAuthUser()
})

describe('ConfirmEmailPage', () => {
  it('shows invalid link when no token', () => {
    renderWithSearch()

    expect(screen.getByText('Invalid Link')).toBeInTheDocument()
    expect(screen.getByText(/invalid or has expired/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /go to settings/i })).toBeInTheDocument()
  })

  it('shows success state after valid token confirmation', async () => {
    renderWithSearch('?token=valid-token')

    await waitFor(() => {
      expect(screen.getByText('Email Updated')).toBeInTheDocument()
    })

    expect(screen.getByText(/changed successfully/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /go to login/i })).toBeInTheDocument()
  })
})
