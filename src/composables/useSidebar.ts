import { ref } from 'vue'
import { api } from '@/api'
import type { Settings } from '@/types'

const isCollapsed = ref(false)
const isInitialized = ref(false)

async function initSidebarState() {
  if (isInitialized.value) return
  try {
    const settings: Settings = await api.getSettings()
    isCollapsed.value = settings.sidebar_collapsed ?? false
  } catch {
    isCollapsed.value = false
  }
  isInitialized.value = true
}

async function toggleSidebar() {
  isCollapsed.value = !isCollapsed.value
  try {
    const settings: Settings = await api.getSettings()
    settings.sidebar_collapsed = isCollapsed.value
    await api.saveSettings(settings)
  } catch (e) {
    console.error('Failed to save sidebar state:', e)
  }
}

async function setSidebarCollapsed(collapsed: boolean) {
  isCollapsed.value = collapsed
  try {
    const settings: Settings = await api.getSettings()
    settings.sidebar_collapsed = collapsed
    await api.saveSettings(settings)
  } catch (e) {
    console.error('Failed to save sidebar state:', e)
  }
}

export function useSidebar() {
  return {
    isCollapsed,
    initSidebarState,
    toggleSidebar,
    setSidebarCollapsed,
  }
}
