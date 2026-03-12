import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import { render } from '@/test/utils'
import { NotFoundPage } from './NotFoundPage'

describe('NotFoundPage', () => {
  it('renders 404 content', () => {
    render(<NotFoundPage />)

    expect(screen.getByText('404')).toBeInTheDocument()
    expect(screen.getByText('Page Not Found')).toBeInTheDocument()
    expect(screen.getByText(/doesn't exist or has been moved/i)).toBeInTheDocument()
  })

  it('shows navigation buttons', () => {
    render(<NotFoundPage />)

    expect(screen.getByRole('button', { name: /go back/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /go home/i })).toBeInTheDocument()
  })
})
