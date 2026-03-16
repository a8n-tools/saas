import type { ApiResponse, ApiError } from '@/types'
import { config } from '@/config'

const API_BASE_URL = config.apiUrl + '/v1'

class ApiClient {
  private baseUrl: string
  private refreshPromise: Promise<void> | null = null

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
    isRetry = false
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`

    const isFormData = options.body instanceof FormData
    const config: RequestInit = {
      ...options,
      credentials: 'include', // Include cookies for auth
      headers: {
        'Accept': 'application/json',
        ...(isFormData ? {} : { 'Content-Type': 'application/json' }),
        ...options.headers,
      },
    }

    const response = await fetch(url, config)

    // Reject non-JSON responses (e.g. Anubis challenge pages)
    const contentType = response.headers.get('content-type') || ''
    if (!contentType.includes('application/json')) {
      throw {
        success: false,
        error: {
          code: 'INVALID_RESPONSE',
          message: 'Server returned a non-JSON response. Please refresh and try again.',
        },
      } satisfies ApiError
    }

    // On 401, try refreshing the token once and retry
    if (
      response.status === 401 &&
      !isRetry &&
      endpoint !== '/auth/refresh' &&
      endpoint !== '/auth/login'
    ) {
      if (!this.refreshPromise) {
        this.refreshPromise = fetch(`${this.baseUrl}/auth/refresh`, {
          method: 'POST',
          credentials: 'include',
        })
          .then((r) => {
            if (!r.ok) throw new Error('refresh failed')
          })
          .finally(() => {
            this.refreshPromise = null
          })
      }

      try {
        await this.refreshPromise
        window.dispatchEvent(new Event('auth:refreshed'))
        return this.request<T>(endpoint, options, true)
      } catch {
        // Refresh failed — fall through to normal error handling
      }
    }

    if (!response.ok) {
      const error: ApiError = await response.json().catch(() => ({
        success: false,
        error: {
          code: 'UNKNOWN_ERROR',
          message: 'An unexpected error occurred',
        },
      }))
      throw error
    }

    const data: ApiResponse<T> = await response.json()
    return data.data
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET' })
  }

  async post<T>(endpoint: string, body?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    })
  }

  async postForm<T>(endpoint: string, body: FormData): Promise<T> {
    return this.request<T>(endpoint, { method: 'POST', body })
  }

  async put<T>(endpoint: string, body?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined,
    })
  }

  async patch<T>(endpoint: string, body?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      body: body ? JSON.stringify(body) : undefined,
    })
  }

  async delete<T>(endpoint: string, body?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'DELETE',
      body: body ? JSON.stringify(body) : undefined,
    })
  }
}

export const apiClient = new ApiClient(API_BASE_URL)
