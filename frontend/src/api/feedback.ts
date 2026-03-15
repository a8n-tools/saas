import { apiClient } from './client'
import type { CreateFeedbackRequest, FeedbackSubmissionResponse } from '@/types'

export interface SubmitFeedbackRequest extends CreateFeedbackRequest {
  files?: File[]
}

export const feedbackApi = {
  submit: (data: SubmitFeedbackRequest): Promise<FeedbackSubmissionResponse> => {
    const formData = new FormData()

    if (data.name) formData.append('name', data.name)
    if (data.email) formData.append('email', data.email)
    if (data.subject) formData.append('subject', data.subject)
    if (data.page_path) formData.append('page_path', data.page_path)
    if (data.website) formData.append('website', data.website)

    formData.append('message', data.message)

    for (const tag of data.tags ?? []) {
      formData.append('tags[]', tag)
    }

    for (const file of data.files ?? []) {
      formData.append('files', file)
    }

    return apiClient.postForm('/feedback', formData)
  },
}
