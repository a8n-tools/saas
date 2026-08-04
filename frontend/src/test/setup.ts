import '@testing-library/jest-dom'
import { afterAll, afterEach, beforeAll, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import { server } from './mocks/server'

// Mock window.matchMedia (not available in jsdom)
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock IntersectionObserver (not available in jsdom)
class MockIntersectionObserver {
  observe = vi.fn()
  unobserve = vi.fn()
  disconnect = vi.fn()
  takeRecords = vi.fn().mockReturnValue([])
  readonly root = null
  readonly rootMargin = '0px'
  readonly thresholds: ReadonlyArray<number> = []
  constructor(_callback: IntersectionObserverCallback, _options?: IntersectionObserverInit) {}
}
Object.defineProperty(window, 'IntersectionObserver', {
  writable: true,
  configurable: true,
  value: MockIntersectionObserver,
})

// Mock navigator.clipboard (not available in jsdom)
Object.defineProperty(navigator, 'clipboard', {
  writable: true,
  configurable: true,
  value: { writeText: vi.fn().mockResolvedValue(undefined) },
})

// Mock URL.createObjectURL / revokeObjectURL (not available in jsdom).
// DEV-525: the avatar preview fetches the image as a blob and turns it into an
// object URL, so without these the settings page can only ever be tested on its
// fallback path.
Object.defineProperty(URL, 'createObjectURL', {
  writable: true,
  configurable: true,
  value: vi.fn(() => 'blob:mock-avatar'),
})
Object.defineProperty(URL, 'revokeObjectURL', {
  writable: true,
  configurable: true,
  value: vi.fn(),
})

// Establish API mocking before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))

// Reset any request handlers that we may add during the tests,
// so they don't affect other tests
afterEach(() => {
  cleanup()
  server.resetHandlers()
  vi.clearAllMocks()
})

// Clean up after the tests are finished
afterAll(() => server.close())
