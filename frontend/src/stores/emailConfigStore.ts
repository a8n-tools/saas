import { create } from 'zustand'

interface EmailConfigState {
  emailEnabled: boolean
  setEmailEnabled: (enabled: boolean) => void
}

export const useEmailConfigStore = create<EmailConfigState>((set) => ({
  emailEnabled: true,
  setEmailEnabled: (enabled) => set({ emailEnabled: enabled }),
}))
