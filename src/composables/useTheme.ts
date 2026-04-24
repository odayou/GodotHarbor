import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark' | 'system'

const currentTheme = ref<Theme>('system')

function applyTheme(theme: Theme) {
  const root = document.documentElement
  const isDark = theme === 'dark' || 
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)

  if (isDark) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
}

export function useTheme() {
  function setTheme(theme: Theme) {
    currentTheme.value = theme
    applyTheme(theme)
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

  return { currentTheme, setTheme, initTheme }
}
