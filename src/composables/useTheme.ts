import { ref, watch } from 'vue'
import { api } from '@/api'

export type Theme = 'light' | 'dark' | 'system' | 'volcano'

const ALL_THEMES: Theme[] = ['light', 'dark', 'system', 'volcano']

const currentTheme = ref<Theme>('system')

function applyTheme(theme: Theme) {
  const root = document.documentElement

  root.classList.remove('dark', 'theme-volcano')

  const isDark = theme === 'dark' ||
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)

  if (isDark) {
    root.classList.add('dark')
  }

  if (theme === 'volcano') {
    root.classList.add('theme-volcano')
  }
}

async function persistTheme(theme: Theme) {
  try {
    const settings = await api.getSettings()
    settings.theme = theme
    await api.saveSettings(settings)
  } catch {}
}

export function useTheme() {
  function setTheme(theme: Theme) {
    currentTheme.value = theme
    applyTheme(theme)
    persistTheme(theme)
  }

  function cycleTheme() {
    const currentIndex = ALL_THEMES.indexOf(currentTheme.value)
    const nextIndex = (currentIndex + 1) % ALL_THEMES.length
    setTheme(ALL_THEMES[nextIndex])
  }

  function initTheme() {
    applyTheme(currentTheme.value)
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (currentTheme.value === 'system') {
        applyTheme('system')
      }
    })
  }

  watch(currentTheme, (newTheme) => {
    applyTheme(newTheme)
  })

  return { currentTheme, setTheme, cycleTheme, initTheme, ALL_THEMES }
}
