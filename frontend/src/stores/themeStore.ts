import { create } from 'zustand'
import { persist } from 'zustand/middleware'

type Theme = 'light' | 'dark' | 'system'

interface ThemeState {
  theme: Theme
  highContrast: boolean
  setTheme: (theme: Theme) => void
  toggleHighContrast: () => void
}

function applyTheme(theme: Theme) {
  const root = window.document.documentElement
  root.classList.remove('light', 'dark')

  if (theme === 'system') {
    const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light'
    root.classList.add(systemTheme)
  } else {
    root.classList.add(theme)
  }
}

function applyHighContrast(enabled: boolean) {
  const root = window.document.documentElement
  root.classList.toggle('high-contrast', enabled)
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      theme: 'system',
      highContrast: false,
      setTheme: (theme) => {
        applyTheme(theme)
        set({ theme })
      },
      toggleHighContrast: () => {
        const next = !get().highContrast
        applyHighContrast(next)
        set({ highContrast: next })
      },
    }),
    {
      name: 'theme-storage',
      onRehydrateStorage: () => (state) => {
        if (state) {
          applyTheme(state.theme)
          applyHighContrast(state.highContrast)
        }
      },
    }
  )
)

// Initialize theme on load
if (typeof window !== 'undefined') {
  const stored = localStorage.getItem('theme-storage')
  if (stored) {
    try {
      const { state } = JSON.parse(stored)
      applyTheme(state.theme || 'system')
      applyHighContrast(state.highContrast || false)
    } catch {
      applyTheme('system')
    }
  } else {
    applyTheme('system')
  }

  // Listen for system theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const currentTheme = useThemeStore.getState().theme
    if (currentTheme === 'system') {
      applyTheme('system')
    }
  })
}
