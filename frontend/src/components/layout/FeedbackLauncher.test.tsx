import { describe, it, expect } from 'vitest'
import { render as rtlRender } from '@testing-library/react'
import { screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { FeedbackLauncher } from './FeedbackLauncher'

function renderAtPath(path: string) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return rtlRender(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[path]}>
        <Routes>
          <Route path="*" element={<FeedbackLauncher />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('FeedbackLauncher', () => {
  it('renders the feedback button on a non-feedback route', () => {
    renderAtPath('/dashboard')
    expect(screen.getByRole('link', { name: /open feedback page/i })).toBeInTheDocument()
  })

  it('renders on the landing page', () => {
    renderAtPath('/')
    expect(screen.getByRole('link', { name: /open feedback page/i })).toBeInTheDocument()
  })

  it('renders on the pricing page', () => {
    renderAtPath('/pricing')
    expect(screen.getByRole('link', { name: /open feedback page/i })).toBeInTheDocument()
  })

  it('returns null on the /feedback route', () => {
    renderAtPath('/feedback')
    expect(screen.queryByRole('link', { name: /open feedback page/i })).not.toBeInTheDocument()
  })

  it('returns null on admin routes', () => {
    renderAtPath('/admin')
    expect(screen.queryByRole('link', { name: /open feedback page/i })).not.toBeInTheDocument()
  })

  it('returns null on admin sub-routes', () => {
    renderAtPath('/admin/users')
    expect(screen.queryByRole('link', { name: /open feedback page/i })).not.toBeInTheDocument()
  })

  it('link points to /feedback', () => {
    renderAtPath('/dashboard')
    const link = screen.getByRole('link', { name: /open feedback page/i })
    expect(link).toHaveAttribute('href', '/feedback')
  })
})
