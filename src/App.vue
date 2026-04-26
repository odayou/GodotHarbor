<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { RouterView } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts'
import { useTheme } from './composables/useTheme'
import { useSidebar } from './composables/useSidebar'
import { useCommandPalette } from './composables/useCommandPalette'
import { usePluginStore } from './stores'
import { useI18n } from 'vue-i18n'
import Sidebar from './components/layout/Sidebar.vue'
import Header from './components/layout/Header.vue'
import StatusBar from './components/layout/StatusBar.vue'
import ToastContainer from './components/ToastContainer.vue'
import OnboardingGuide from './components/OnboardingGuide.vue'
import CommandPalette from './components/CommandPalette.vue'

const { t, locale } = useI18n()
const showLanguageDialog = ref(false)

const pluginStore = usePluginStore()
let unlistenProgress: any = null

const { registerShortcut } = useKeyboardShortcuts()
const { currentTheme, setTheme } = useTheme()
const { initSidebarState, toggleSidebar } = useSidebar()
const { openPalette } = useCommandPalette()

getCurrentWindow().show().catch((e) => {
  console.error('Failed to show window:', e)
})

onMounted(async () => {
  initSidebarState()
  
  // 检查是否首次启动，显示语言选择提示
  const hasSetLanguage = localStorage.getItem('godotharbor-language')
  const hasLaunchedBefore = localStorage.getItem('godotharbor-launched')
  
  if (!hasSetLanguage && !hasLaunchedBefore) {
    showLanguageDialog.value = true
  }
  
  // 标记已启动
  localStorage.setItem('godotharbor-launched', 'true')
  
  // 监听资产导入进度事件
  unlistenProgress = await listen('asset-import-progress', (event) => {
    pluginStore.setImportProgress(event.payload)
  })
})

const selectLanguage = (lang: string) => {
  locale.value = lang
  localStorage.setItem('godotharbor-language', lang)
  showLanguageDialog.value = false
}

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress()
  }
})

registerShortcut({
  key: 'b',
  ctrl: true,
  handler: () => {
    toggleSidebar()
  },
  description: t('sidebar.toggleShortcut')
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
  description: t('sidebar.toggleThemeShortcut')
})

registerShortcut({
  key: 'd',
  ctrl: true,
  handler: () => {
    setTheme(currentTheme.value === 'dark' ? 'light' : 'dark')
  },
  description: t('sidebar.toggleThemeModeShortcut')
})

registerShortcut({
  key: 'k',
  ctrl: true,
  handler: () => {
    openPalette()
  },
  description: t('sidebar.openCommandPaletteShortcut'),
  global: true
})
</script>

<template>
  <div class="flex h-screen bg-white dark:bg-surface-layer" @contextmenu.prevent>
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
  
  <!-- 首次启动语言选择对话框 -->
  <div v-if="showLanguageDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
      <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">选择语言 / Select Language</h3>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-6">请选择您偏好的语言 / Please select your preferred language</p>
      <div class="space-y-3">
        <button
          @click="selectLanguage('zh-CN')"
          class="w-full px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
        >
          简体中文
        </button>
        <button
          @click="selectLanguage('en')"
          class="w-full px-4 py-3 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors"
        >
          English
        </button>
      </div>
    </div>
  </div>
</template>