import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark' | 'system' | 'volcano'

const currentTheme = ref<Theme>('system')

function applyTheme(theme: Theme) {
  const root = document.documentElement
  
  // 重置所有主题类
  root.classList.remove('dark', 'theme-volcano')
  
  // 应用深色模式
  const isDark = theme === 'dark' || 
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)

  if (isDark) {
    root.classList.add('dark')
  }
  
  // 应用火山引擎主题
  if (theme === 'volcano') {
    root.classList.add('theme-volcano')
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