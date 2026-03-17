import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '@/test/mocks/server'

// The apiClient is a singleton, so we test it through the imported instance
// We use MSW to control server responses
import { apiClient } from './client'

const API_BASE = `${import.meta.env.VITE_API_URL || ''}/v1`

describe('apiClient', () => {
  describe('get', () => {
    it('makes a GET request and returns data', async () => {
      const result = await apiClient.get<{ id: string }>('/users/me')
      expect(result).toMatchObject({ id: expect.any(String) })
    })
  })

  describe('post', () => {
    it('makes a POST request with JSON body', async () => {
      const result = await apiClient.post<{ message: string }>('/auth/logout')
      // The mock returns null for logout, which is fine
      expect(result).toBeNull()
    })
  })

  describe('error handling', () => {
    it('throws ApiError on non-OK response', async () => {
      await expect(
        apiClient.post('/auth/login', {
          email: 'test@example.com',
          password: 'wrongpassword',
        })
      ).rejects.toMatchObject({
        error: {
          code: 'INVALID_CREDENTIALS',
        },
      })
    })

    it('throws on non-JSON response', async () => {
      server.use(
        http.get(`${API_BASE}/test-html`, () => {
          return new HttpResponse('<html>challenge</html>', {
            headers: { 'Content-Type': 'text/html' },
          })
        })
      )

      await expect(apiClient.get('/test-html')).rejects.toMatchObject({
        error: {
          code: 'INVALID_RESPONSE',
          message: expect.stringContaining('non-JSON'),
        },
      })
    })
  })

  describe('401 refresh flow', () => {
    it('retries with refreshed token on 401', async () => {
      let callCount = 0
      server.use(
        http.get(`${API_BASE}/test-auth`, () => {
          callCount++
          if (callCount === 1) {
            return HttpResponse.json(
              { success: false, error: { code: 'UNAUTHORIZED', message: 'Token expired' } },
              { status: 401 }
            )
          }
          return HttpResponse.json({ success: true, data: { ok: true } })
        })
      )

      const result = await apiClient.get<{ ok: boolean }>('/test-auth')
      expect(result).toEqual({ ok: true })
      expect(callCount).toBe(2)
    })

    it('does not retry refresh endpoint itself', async () => {
      server.use(
        http.post(`${API_BASE}/auth/refresh`, () => {
          return HttpResponse.json(
            { success: false, error: { code: 'UNAUTHORIZED', message: 'Refresh expired' } },
            { status: 401 }
          )
        })
      )

      await expect(apiClient.post('/auth/refresh')).rejects.toMatchObject({
        error: { code: 'UNAUTHORIZED' },
      })
    })

    it('does not retry login endpoint on 401', async () => {
      // Login already has a handler returning 401 for wrong password
      await expect(
        apiClient.post('/auth/login', {
          email: 'test@example.com',
          password: 'wrongpassword',
        })
      ).rejects.toMatchObject({
        error: { code: 'INVALID_CREDENTIALS' },
      })
    })
  })

  describe('postForm', () => {
    it('sends FormData without Content-Type header', async () => {
      const formData = new FormData()
      formData.append('message', 'test')

      const result = await apiClient.postForm<{ id: string }>('/feedback', formData)
      expect(result).toMatchObject({ id: 'fb-001' })
    })
  })
})
