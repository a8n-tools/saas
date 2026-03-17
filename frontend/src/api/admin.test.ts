import { describe, it, expect } from 'vitest'
import { adminApi, getFeedbackAttachmentUrl } from './admin'
import {
  mockAdminStats,
  mockAdminUser,
  mockAdminApplication,
  mockAdminMembership,
  mockAuditLog,
  mockFeedbackSummary,
  mockFeedbackDetail,
  mockUser,
} from '@/test/mocks/handlers'

describe('adminApi', () => {
  describe('getStats', () => {
    it('returns admin stats', async () => {
      const stats = await adminApi.getStats()
      expect(stats).toEqual(mockAdminStats)
    })
  })

  describe('getUsers', () => {
    it('returns paginated users', async () => {
      const result = await adminApi.getUsers()
      expect(result.items).toHaveLength(1)
      expect(result.items[0].email).toBe(mockUser.email)
      expect(result.total).toBe(1)
    })
  })

  describe('updateUserStatus', () => {
    it('updates user status', async () => {
      const result = await adminApi.updateUserStatus('user-1', { is_active: false })
      expect(result.email).toBe(mockAdminUser.email)
    })
  })

  describe('updateUserRole', () => {
    it('updates user role', async () => {
      const result = await adminApi.updateUserRole('user-1', { role: 'admin' })
      expect(result.email).toBe(mockAdminUser.email)
    })
  })

  describe('deleteUser', () => {
    it('deletes user without error', async () => {
      await expect(adminApi.deleteUser('user-1')).resolves.not.toThrow()
    })
  })

  describe('resetUserPassword', () => {
    it('returns temporary password', async () => {
      const result = await adminApi.resetUserPassword('user-1')
      expect(result.temporary_password).toBe('TempPass123!')
    })
  })

  describe('getMemberships', () => {
    it('returns paginated memberships', async () => {
      const result = await adminApi.getMemberships()
      expect(result.items).toHaveLength(1)
      expect(result.items[0]).toMatchObject({
        id: mockAdminMembership.id,
        status: 'active',
      })
    })
  })

  describe('revokeMembership', () => {
    it('revokes membership', async () => {
      const result = await adminApi.revokeMembership({ user_id: 'user-1' })
      expect(result.message).toBe('Revoked')
    })
  })

  describe('getApplications', () => {
    it('returns list of applications', async () => {
      const apps = await adminApi.getApplications()
      expect(Array.isArray(apps)).toBe(true)
      expect(apps[0]).toMatchObject({
        id: mockAdminApplication.id,
        slug: mockAdminApplication.slug,
      })
    })
  })

  describe('updateApplication', () => {
    it('updates application', async () => {
      const result = await adminApi.updateApplication('app-1', { display_name: 'Updated' })
      expect(result.slug).toBe(mockAdminApplication.slug)
    })
  })

  describe('getAuditLogs', () => {
    it('returns paginated audit logs', async () => {
      const result = await adminApi.getAuditLogs()
      expect(result.items).toHaveLength(1)
      expect(result.items[0]).toMatchObject({
        id: mockAuditLog.id,
        action: mockAuditLog.action,
      })
    })
  })

  describe('getFeedback', () => {
    it('returns paginated feedback', async () => {
      const result = await adminApi.getFeedback()
      expect(result.items).toHaveLength(1)
      expect(result.items[0]).toMatchObject({
        id: mockFeedbackSummary.id,
        subject: mockFeedbackSummary.subject,
      })
    })
  })

  describe('getFeedbackDetail', () => {
    it('returns feedback detail', async () => {
      const result = await adminApi.getFeedbackDetail('fb-001')
      expect(result).toMatchObject({
        id: mockFeedbackDetail.id,
        message: mockFeedbackDetail.message,
      })
    })
  })

  describe('respondToFeedback', () => {
    it('responds to feedback', async () => {
      const result = await adminApi.respondToFeedback('fb-001', {
        response: 'Thank you!',
      })
      expect(result.status).toBe('responded')
      expect(result.admin_response).toBe('Thank you for your feedback!')
    })
  })

  describe('updateFeedbackStatus', () => {
    it('updates feedback status', async () => {
      const result = await adminApi.updateFeedbackStatus('fb-001', 'closed')
      expect(result.status).toBe('closed')
    })
  })

  describe('deleteFeedback', () => {
    it('deletes feedback without error', async () => {
      await expect(adminApi.deleteFeedback('fb-001')).resolves.not.toThrow()
    })
  })

  describe('getArchivedFeedback', () => {
    it('returns paginated archived feedback', async () => {
      const result = await adminApi.getArchivedFeedback()
      expect(result.items).toEqual([])
      expect(result.total).toBe(0)
    })
  })

  describe('getUser', () => {
    it('returns a single user', async () => {
      const result = await adminApi.getUser('user-1')
      expect(result.email).toBe(mockUser.email)
    })
  })

  describe('impersonateUser', () => {
    it('starts impersonation', async () => {
      const result = await adminApi.impersonateUser('user-1')
      expect(result.message).toBe('Impersonation started')
    })
  })

  describe('createApplication', () => {
    it('creates a new application', async () => {
      const result = await adminApi.createApplication({
        name: 'new-app',
        slug: 'new-app',
        display_name: 'New App',
        container_name: 'new-app',
      })
      expect(result.id).toBe('app-new')
    })
  })

  describe('deleteApplication', () => {
    it('deletes an application', async () => {
      await expect(
        adminApi.deleteApplication('app-1', { password: 'pass', totp_code: '123456' })
      ).resolves.not.toThrow()
    })
  })

  describe('swapApplicationOrder', () => {
    it('swaps application order', async () => {
      const result = await adminApi.swapApplicationOrder('app-1', 'app-2')
      expect(Array.isArray(result)).toBe(true)
    })
  })

  describe('grantMembership', () => {
    it('grants membership to a user', async () => {
      const result = await adminApi.grantMembership({ user_id: 'user-1', tier: 'personal' })
      expect(result.status).toBe('active')
    })
  })

  describe('getNotifications', () => {
    it('returns notifications list', async () => {
      const result = await adminApi.getNotifications()
      expect(Array.isArray(result)).toBe(true)
    })
  })

  describe('markNotificationRead', () => {
    it('marks a notification as read', async () => {
      await expect(adminApi.markNotificationRead('notif-1')).resolves.not.toThrow()
    })
  })

  describe('markAllNotificationsRead', () => {
    it('marks all notifications as read', async () => {
      await expect(adminApi.markAllNotificationsRead()).resolves.not.toThrow()
    })
  })

  describe('restoreFeedback', () => {
    it('restores archived feedback', async () => {
      const result = await adminApi.restoreFeedback('archive-1')
      expect(result.status).toBe('reviewed')
    })
  })

  describe('getHealth', () => {
    it('returns health status', async () => {
      const result = await adminApi.getHealth()
      expect(result).toEqual({
        status: 'healthy',
        database: 'healthy',
        uptime_seconds: 3600,
      })
    })
  })
})

describe('getFeedbackAttachmentUrl', () => {
  it('constructs the correct URL', () => {
    const url = getFeedbackAttachmentUrl('fb-001', 'att-001')
    expect(url).toContain('/v1/admin/feedback/fb-001/attachments/att-001')
  })
})
