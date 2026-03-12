import { describe, it, expect, beforeEach, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, setupAuthUser } from '@/test/utils'
import { TwoFactorSetupPage } from './TwoFactorSetupPage'

// Mock qrcode.react since jsdom doesn't support SVG rendering
vi.mock('qrcode.react', () => ({
  QRCodeSVG: ({ value }: { value: string }) => <div data-testid="qr-code" data-value={value} />,
}))

beforeEach(() => {
  setupAuthUser()

  Object.defineProperty(navigator, 'clipboard', {
    configurable: true,
    writable: true,
    value: { writeText: vi.fn().mockResolvedValue(undefined) },
  })
})

describe('TwoFactorSetupPage', () => {
  it('renders setup heading', () => {
    render(<TwoFactorSetupPage />)

    expect(screen.getByText('Set Up Two-Factor Authentication')).toBeInTheDocument()
  })

  it('shows loading spinner while fetching QR code', () => {
    render(<TwoFactorSetupPage />)

    // Initially loading
    expect(document.querySelector('.animate-spin')).toBeInTheDocument()
  })

  it('shows QR code and secret after setup loads', async () => {
    render(<TwoFactorSetupPage />)

    await waitFor(() => {
      expect(screen.getByTestId('qr-code')).toBeInTheDocument()
      expect(screen.getByText('TESTSECRET')).toBeInTheDocument()
    })
  })

  it('shows step 1 scan heading', async () => {
    render(<TwoFactorSetupPage />)

    await waitFor(() => {
      expect(screen.getByText('Step 1: Scan QR Code')).toBeInTheDocument()
    })
  })

  it('shows continue button to proceed to verify step', async () => {
    const user = userEvent.setup()
    render(<TwoFactorSetupPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /continue/i })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /continue/i }))

    expect(screen.getByText('Step 2: Verify Code')).toBeInTheDocument()
  })

  it('shows verify step with verification code input', async () => {
    const user = userEvent.setup()
    render(<TwoFactorSetupPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /continue/i })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /continue/i }))

    expect(screen.getByLabelText('Verification Code')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /verify & enable/i })).toBeInTheDocument()
  })

  it('goes back to scan step from verify step', async () => {
    const user = userEvent.setup()
    render(<TwoFactorSetupPage />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /continue/i })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: /continue/i }))
    await user.click(screen.getByRole('button', { name: /back/i }))

    expect(screen.getByText('Step 1: Scan QR Code')).toBeInTheDocument()
  })
})
