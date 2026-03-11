import { describe, it, expect, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { render as rtlRender } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { VerifyEmailPage } from './VerifyEmailPage'
import { useAuthStore } from '@/stores/authStore'
import { mockUser } from '@/test/mocks/handlers'

function renderWithSearch(search = '') {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return rtlRender(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[`/settings/verify-email${search}`]}>
        <Routes>
          <Route path="/settings/verify-email" element={<VerifyEmailPage />} />
          <Route path="/settings" element={<div>Settings Page</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

beforeEach(() => {
  useAuthStore.setState({
    user: mockUser,
    isAuthenticated: true,
    isLoading: false,
    error: null,
    pendingChallenge: null,
  })
})

describe('VerifyEmailPage', () => {
  it('shows invalid link state when no token', () => {
    renderWithSearch()

    expect(screen.getByText('Invalid Link')).toBeInTheDocument()
    expect(screen.getByText(/invalid or has expired/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /go to settings/i })).toBeInTheDocument()
  })

  it('shows success state after valid token verification', async () => {
    renderWithSearch('?token=valid-verify-token')

    await waitFor(() => {
      expect(screen.getByText('Email Verified!')).toBeInTheDocument()
    })

    expect(screen.getByText(/verified successfully/i)).toBeInTheDocument()
  })

  it('shows error state after invalid token', async () => {
    renderWithSearch('?token=invalid-token')

    await waitFor(() => {
      expect(screen.getByText('Verification Failed')).toBeInTheDocument()
    })
  })
})
