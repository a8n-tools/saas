import { describe, it, expect } from 'vitest'
import { applicationApi } from './applications'
import { mockApplication } from '@/test/mocks/handlers'

describe('applicationApi', () => {
  describe('list', () => {
    it('returns list of applications', async () => {
      const applications = await applicationApi.list()

      expect(Array.isArray(applications)).toBe(true)
      expect(applications[0]).toMatchObject({
        slug: mockApplication.slug,
        display_name: mockApplication.display_name,
      })
    })
  })

  describe('get', () => {
    it('returns a single application by slug', async () => {
      const application = await applicationApi.get('rus')

      expect(application.slug).toBe(mockApplication.slug)
      expect(application.display_name).toBe(mockApplication.display_name)
    })
  })
})
