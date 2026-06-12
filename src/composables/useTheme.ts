import { ref, watch } from 'vue'
import { api } from '@/api'

export type Theme = 'light' | 'dark' | 'system'
export type Density = 'default' | 'compact'

const ALL_THEMES: Theme[] = ['light', 'dark', 'system']

const currentTheme = ref<Theme>('system')
const currentDensity = ref<Density>('default')

function applyTheme(theme: Theme) {
  const root = document.documentElement

  root.classList.remove('dark')

  const isDark = theme === 'dark' ||
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)

  if (isDark) {
    root.classList.add('dark')
  }
}

function applyDensity(density: Density) {
  const root = document.documentElement

  root.classList.remove('compact')

  if (density === 'compact') {
    root.classList.add('compact')
  }
}

async function persistTheme(theme: Theme) {
  try {
    const settings = await api.getSettings()
    settings.theme = theme
    await api.saveSettings(settings)
  } catch {}
}

async function persistDensity(density: Density) {
  try {
    const settings = await api.getSettings()
    settings.density = density
    await api.saveSettings(settings)
  } catch {}
}

export function useTheme() {
  function setTheme(theme: Theme) {
    currentTheme.value = theme
    applyTheme(theme)
    persistTheme(theme)
  }

  function setDensity(density: Density) {
    currentDensity.value = density
    applyDensity(density)
    persistDensity(density)
  }

  function toggleDensity() {
    const next = currentDensity.value === 'default' ? 'compact' : 'default'
    setDensity(next)
  }

  function cycleTheme() {
    const currentIndex = ALL_THEMES.indexOf(currentTheme.value)
    const nextIndex = (currentIndex + 1) % ALL_THEMES.length
    setTheme(ALL_THEMES[nextIndex])
  }

  async function initTheme() {
    applyTheme(currentTheme.value)
    applyDensity(currentDensity.value)
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (currentTheme.value === 'system') {
        applyTheme('system')
      }
    })
    try {
      const settings = await api.getSettings()
      if (['light', 'dark', 'system'].includes(settings.theme)) {
        currentTheme.value = settings.theme as Theme
        applyTheme(currentTheme.value)
      }
      if (settings.density === 'compact' || settings.density === 'default') {
        currentDensity.value = settings.density as Density
        applyDensity(currentDensity.value)
      }
    } catch {}
  }

  watch(currentTheme, (newTheme) => {
    applyTheme(newTheme)
  })

  watch(currentDensity, (newDensity) => {
    applyDensity(newDensity)
  })

  return {
    currentTheme,
    currentDensity,
    setTheme,
    setDensity,
    toggleDensity,
    cycleTheme,
    initTheme,
    ALL_THEMES,
  }
}
