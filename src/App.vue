<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { RouterView, useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts'
import { useTheme } from './composables/useTheme'
import { useSidebar } from './composables/useSidebar'
import { useCommandPalette } from './composables/useCommandPalette'
import { usePluginStore } from './stores'
import { useI18n } from 'vue-i18n'
import { useLanguageDialog } from './composables/useLanguageDialog'
import { api } from './api'
import Sidebar from './components/layout/Sidebar.vue'
import Header from './components/layout/Header.vue'
import StatusBar from './components/layout/StatusBar.vue'
import ToastContainer from './components/ToastContainer.vue'
import OnboardingGuide from './components/OnboardingGuide.vue'
import DataDirSetupDialog from './components/DataDirSetupDialog.vue'
import CommandPalette from './components/CommandPalette.vue'
import { useAutoSetup } from './composables/useAutoSetup'
import { useNetworkStatus } from './composables/useNetworkStatus'
import { useToast } from './composables/useToast'

const { t, locale } = useI18n()
const router = useRouter()
const { isVisible: showLanguageDialog, hideLanguageDialog } = useLanguageDialog()
const { isOnline } = useNetworkStatus()
const toast = useToast()

const pluginStore = usePluginStore()
const { runAutoSetup, isRunning: isAutoSetupRunning } = useAutoSetup()
const showDataDirSetup = ref(false)
let unlistenProgress: any = null
let unlistenScanComplete: any = null
let unlistenEnginesDiscovered: any = null

const { registerShortcut } = useKeyboardShortcuts()
const { currentTheme, setTheme, cycleTheme } = useTheme()
const { initSidebarState, toggleSidebar } = useSidebar()
const { openPalette } = useCommandPalette()

getCurrentWindow().show().then(() => {
  getCurrentWindow().setAlwaysOnTop(true).then(() => {
    setTimeout(() => {
      getCurrentWindow().setAlwaysOnTop(false).catch(() => {})
    }, 100)
  }).catch(() => {})
}).catch((e) => {
  console.error('Failed to show window:', e)
})

onMounted(async () => {
  initSidebarState()
  
  try {
    const needSetup = await api.checkDataDirSetupNeeded()
    if (needSetup) {
      showDataDirSetup.value = true
    }
  } catch {}

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

  unlistenScanComplete = await listen('scan-complete', async () => {
    if (!isAutoSetupRunning.value) {
      await runAutoSetup(undefined, true, true)
    }
  })

  unlistenEnginesDiscovered = await listen<{ engine_id: string; name: string }[]>('engines-discovered', (event) => {
    const count = event.payload.length
    if (count > 0) {
      toast.success(t('engines.discovered', { count }))
    }
  })
})

const selectLanguage = async (lang: string) => {
  locale.value = lang
  localStorage.setItem('godotharbor-language', lang)
  hideLanguageDialog()
  try {
    const currentSettings = await api.getSettings()
    currentSettings.language = lang
    await api.saveSettings(currentSettings)
  } catch {}
}

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress()
  }
  if (unlistenScanComplete) {
    unlistenScanComplete()
  }
  if (unlistenEnginesDiscovered) {
    unlistenEnginesDiscovered()
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
    cycleTheme()
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

registerShortcut({
  key: '1',
  ctrl: true,
  handler: () => {
    router.push('/')
  },
  description: t('sidebar.navHomeShortcut')
})

registerShortcut({
  key: '2',
  ctrl: true,
  handler: () => {
    router.push('/projects')
  },
  description: t('sidebar.navProjectsShortcut')
})

registerShortcut({
  key: '3',
  ctrl: true,
  handler: () => {
    router.push('/plugins')
  },
  description: t('sidebar.navPluginsShortcut')
})

registerShortcut({
  key: '4',
  ctrl: true,
  handler: () => {
    router.push('/engines')
  },
  description: t('sidebar.navEnginesShortcut')
})
</script>

<template>
  <div class="flex h-screen bg-surface-light-base dark:bg-surface-base border border-gray-200/60 dark:border-surface-border/40" @contextmenu.prevent>
    <Sidebar />
    <div class="flex-1 flex flex-col overflow-hidden">
      <div v-if="!isOnline" class="bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300 text-xs text-center py-1.5 px-4">
        {{ t('common.offlineNotice') }}
      </div>
      <Header />
      <main class="flex-1 overflow-x-hidden overflow-y-auto bg-surface-light-base dark:bg-surface-base p-4">
        <RouterView v-slot="{ Component }">
          <component :is="Component" />
        </RouterView>
      </main>
      <StatusBar />
    </div>
    <ToastContainer />
    <OnboardingGuide />
    <DataDirSetupDialog v-model:visible="showDataDirSetup" />
    <CommandPalette />
  </div>

  <!-- 首次启动语言选择对话框 -->
  <div v-if="showLanguageDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[110]">
    <div class="dialog-container w-full max-w-md" @click.stop>
      <h3 class="dialog-title">选择语言 / Select Language</h3>
      <p class="text-sm text-gray-500 dark:text-content-muted mb-6">请选择您偏好的语言 / Please select your preferred language</p>
      <p class="text-xs text-gray-400 dark:text-content-muted mb-4">可在「设置 &rarr; 外观」中切换 / Switch in Settings &rarr; Appearance</p>
      <div class="space-y-3">
        <button
          @click="selectLanguage('zh-CN')"
          class="btn-primary w-full px-4 py-3"
        >
          简体中文
        </button>
        <button
          @click="selectLanguage('en')"
          class="btn-secondary w-full px-4 py-3"
        >
          English
        </button>
      </div>
    </div>
  </div>
</template>