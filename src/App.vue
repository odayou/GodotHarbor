<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts'
import { useTheme } from './composables/useTheme'
import { useSidebar } from './composables/useSidebar'
import { useCommandPalette } from './composables/useCommandPalette'
import Sidebar from './components/layout/Sidebar.vue'
import Header from './components/layout/Header.vue'
import StatusBar from './components/layout/StatusBar.vue'
import ToastContainer from './components/ToastContainer.vue'
import OnboardingGuide from './components/OnboardingGuide.vue'
import CommandPalette from './components/CommandPalette.vue'

const { registerShortcut } = useKeyboardShortcuts()
const { currentTheme, setTheme } = useTheme()
const { initSidebarState, toggleSidebar } = useSidebar()
const { openPalette } = useCommandPalette()

getCurrentWindow().show().catch((e) => {
  console.error('Failed to show window:', e)
})

onMounted(() => {
  initSidebarState()
})

registerShortcut({
  key: 'b',
  ctrl: true,
  handler: () => {
    toggleSidebar()
  },
  description: '切换侧边栏折叠'
})

registerShortcut({
  key: 't',
  ctrl: true,
  handler: () => {
    const themes = ['light', 'dark', 'system'] as const
    const currentIndex = themes.indexOf(currentTheme.value as typeof themes[number])
    const nextIndex = (currentIndex + 1) % themes.length
    setTheme(themes[nextIndex])
  },
  description: '切换主题'
})

registerShortcut({
  key: 'd',
  ctrl: true,
  handler: () => {
    setTheme(currentTheme.value === 'dark' ? 'light' : 'dark')
  },
  description: '切换深色/浅色模式'
})

registerShortcut({
  key: 'k',
  ctrl: true,
  handler: () => {
    openPalette()
  },
  description: '打开命令面板',
  global: true
})
</script>

<template>
  <div class="flex h-screen bg-white dark:bg-surface-layer">
    <Sidebar />
    <div class="flex-1 flex flex-col overflow-hidden">
      <Header />
      <main class="flex-1 overflow-x-hidden overflow-y-auto bg-white dark:bg-surface-layer p-4 md:p-6">
        <RouterView />
      </main>
      <StatusBar />
    </div>
    <ToastContainer />
    <OnboardingGuide />
    <CommandPalette />
  </div>
</template>