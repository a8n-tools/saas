import { describe, it, expect } from 'vitest'
import { feedbackApi } from './feedback'

describe('feedbackApi', () => {
  describe('submit', () => {
    it('submits feedback successfully', async () => {
      const result = await feedbackApi.submit({
        message: 'Great app!',
      })

      expect(result.id).toBe('fb-001')
      expect(result.message).toBe('Feedback submitted')
    })

    it('submits feedback with optional fields', async () => {
      const result = await feedbackApi.submit({
        name: 'Alice',
        email: 'alice@example.com',
        subject: 'Bug report',
        tags: ['Bug', 'UI'],
        message: 'Found a bug',
        page_path: '/dashboard',
      })

      expect(result.id).toBe('fb-001')
    })

    it('submits feedback with files', async () => {
      const file = new File(['screenshot'], 'screenshot.png', { type: 'image/png' })

      const result = await feedbackApi.submit({
        message: 'See attached',
        files: [file],
      })

      expect(result.id).toBe('fb-001')
    })
  })
})
