<script setup lang="ts">
import { RouterView } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts'
import { useTheme } from './composables/useTheme'
import Sidebar from './components/layout/Sidebar.vue'
import Header from './components/layout/Header.vue'
import StatusBar from './components/layout/StatusBar.vue'
import ToastContainer from './components/ToastContainer.vue'
import OnboardingGuide from './components/OnboardingGuide.vue'

const { registerShortcut } = useKeyboardShortcuts()
const { currentTheme, setTheme } = useTheme()

getCurrentWindow().show().catch((e) => {
  console.error('Failed to show window:', e)
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
</script>

<template>
  <div class="flex h-screen bg-gray-50 dark:bg-gray-900">
    <Sidebar />
    <div class="flex-1 flex flex-col overflow-hidden">
      <Header />
      <main class="flex-1 overflow-x-hidden overflow-y-auto bg-gray-50 dark:bg-gray-900 p-4 md:p-6">
        <RouterView />
      </main>
      <StatusBar />
    </div>
    <ToastContainer />
    <OnboardingGuide />
  </div>
</template>