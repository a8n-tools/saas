import { describe, it, expect, beforeEach } from 'vitest'
import { useThemeStore } from './themeStore'

beforeEach(() => {
  useThemeStore.setState({ theme: 'system' })
  document.documentElement.classList.remove('light', 'dark')
})

describe('themeStore', () => {
  describe('initial state', () => {
    it('starts with system theme', () => {
      const { theme } = useThemeStore.getState()
      expect(theme).toBe('system')
    })
  })

  describe('setTheme', () => {
    it('sets light theme', () => {
      const { setTheme } = useThemeStore.getState()
      setTheme('light')

      expect(useThemeStore.getState().theme).toBe('light')
      expect(document.documentElement.classList.contains('light')).toBe(true)
    })

    it('sets dark theme', () => {
      const { setTheme } = useThemeStore.getState()
      setTheme('dark')

      expect(useThemeStore.getState().theme).toBe('dark')
      expect(document.documentElement.classList.contains('dark')).toBe(true)
    })

    it('sets system theme', () => {
      const { setTheme } = useThemeStore.getState()
      setTheme('light')
      setTheme('system')

      expect(useThemeStore.getState().theme).toBe('system')
    })

    it('removes previous theme class when switching', () => {
      const { setTheme } = useThemeStore.getState()
      setTheme('dark')
      setTheme('light')

      expect(document.documentElement.classList.contains('dark')).toBe(false)
      expect(document.documentElement.classList.contains('light')).toBe(true)
    })
  })
})
