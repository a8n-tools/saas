import { apiClient } from './client'
import type { CreateFeedbackRequest, FeedbackSubmissionResponse } from '@/types'

export const feedbackApi = {
  submit: (data: CreateFeedbackRequest): Promise<FeedbackSubmissionResponse> =>
    apiClient.post('/feedback', data),
}
