import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api'
import type { AppUpdateInfo, PluginUpdateInfo, VersionUpdateInfo, HotUpdateInfo, UpdateHistoryEntry, UpdateProgress } from '@/types'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

export const useUpdateStore = defineStore('updates', () => {
  const { t } = useI18n()
  const isChecking = ref(false)
  const lastCheckedAt = ref('')

  const appUpdate = ref<AppUpdateInfo | null>(null)
  const pluginUpdates = ref<PluginUpdateInfo[]>([])
  const engineUpdates = ref<VersionUpdateInfo[]>([])
  const hotUpdate = ref<HotUpdateInfo | null>(null)
  const currentHotUpdateVersion = ref<string | null>(null)
  const updateHistory = ref<UpdateHistoryEntry[]>([])

  const isInstallingApp = ref(false)
  const isUpdatingPlugins = ref(false)
  const isInstallingHotUpdate = ref(false)
  const installProgress = ref(0)
  const installMessage = ref('')
  const hotUpdateProgress = ref(0)
  const hotUpdateMessage = ref('')

  const updateProgress = ref<UpdateProgress | null>(null)

  const totalUpdateCount = computed(() => {
    let count = 0
    if (appUpdate.value) count++
    count += pluginUpdates.value.length
    count += engineUpdates.value.length
    if (hotUpdate.value) count++
    return count
  })

  const hasAnyUpdate = computed(() => totalUpdateCount.value > 0)

  const unlisteners: (() => void)[] = []

  async function initListeners() {
    const unlisten1 = await listen('app-update-progress', (event: any) => {
      installProgress.value = event.payload.progress || 0
      installMessage.value = event.payload.message || ''
      updateProgress.value = {
        update_type: 'app',
        target_id: '',
        stage: event.payload.stage || '',
        progress: event.payload.progress || 0,
        message: event.payload.message || ''
      }
    })
    const unlisten2 = await listen('updates-available', () => {
      checkAll()
    })
    const unlisten3 = await listen('hot-update-progress', (event: any) => {
      hotUpdateProgress.value = event.payload.progress || 0
      hotUpdateMessage.value = event.payload.message || ''
      updateProgress.value = {
        update_type: 'hot',
        target_id: '',
        stage: event.payload.stage || '',
        progress: event.payload.progress || 0,
        message: event.payload.message || ''
      }
    })
    const unlisten4 = await listen('plugin-update-progress', (event: any) => {
      updateProgress.value = {
        update_type: 'plugin',
        target_id: event.payload.plugin_id || '',
        stage: event.payload.stage || '',
        progress: event.payload.progress || 0,
        message: event.payload.message || ''
      }
    })
    unlisteners.push(unlisten1, unlisten2, unlisten3, unlisten4)
  }

  function cleanupListeners() {
    unlisteners.forEach(fn => fn())
    unlisteners.length = 0
  }

  async function checkAll() {
    if (isChecking.value) return
    isChecking.value = true
    try {
      const result = await api.checkAllUpdates()
      appUpdate.value = result.app_update
      pluginUpdates.value = result.plugin_updates
      engineUpdates.value = result.engine_updates
      lastCheckedAt.value = result.checked_at

      if (!appUpdate.value) {
        try {
          const appUpd = await api.checkAppUpdate()
          appUpdate.value = appUpd
        } catch {}
      }

      try {
        const hotUpd = await api.checkHotUpdate()
        hotUpdate.value = hotUpd
      } catch {}

      try {
        currentHotUpdateVersion.value = await api.getCurrentHotUpdateVersion()
      } catch {}

      try {
        updateHistory.value = await api.getUpdateHistory()
      } catch {}
    } catch (error) {
      console.error('Check updates failed:', error)
    } finally {
      isChecking.value = false
    }
  }

  async function installAppUpdate() {
    isInstallingApp.value = true
    installProgress.value = 0
    installMessage.value = t('updates.preparingDownload')
    try {
      await api.installAppUpdate()
      installMessage.value = t('updates.installCompleteRestarting')
    } catch (error) {
      installMessage.value = t('updates.installFailed', { error })
      throw error
    } finally {
      isInstallingApp.value = false
    }
  }

  async function skipAppVersion() {
    if (appUpdate.value) {
      await api.skipAppVersion(appUpdate.value.latest_version)
      appUpdate.value = null
    }
  }

  async function updateSinglePlugin(pluginId: string) {
    await api.updateGitPlugin(pluginId)
    pluginUpdates.value = pluginUpdates.value.filter(u => u.plugin_id !== pluginId)
  }

  async function batchUpdateAllPlugins() {
    isUpdatingPlugins.value = true
    try {
      const ids = pluginUpdates.value.map(u => u.plugin_id)
      const result = await api.batchUpdatePlugins(ids)
      if (result.success_count > 0) {
        pluginUpdates.value = []
      }
    } finally {
      isUpdatingPlugins.value = false
    }
  }

  async function installHotUpdate() {
    isInstallingHotUpdate.value = true
    hotUpdateProgress.value = 0
    hotUpdateMessage.value = t('updates.preparingHotUpdate')
    try {
      await api.installHotUpdate()
      hotUpdate.value = null
      currentHotUpdateVersion.value = await api.getCurrentHotUpdateVersion()
    } catch (error) {
      hotUpdateMessage.value = t('updates.hotUpdateFailed', { error })
      throw error
    } finally {
      isInstallingHotUpdate.value = false
    }
  }

  async function rollbackHotUpdate() {
    await api.rollbackHotUpdate()
    currentHotUpdateVersion.value = null
  }

  async function loadHistory() {
    try {
      updateHistory.value = await api.getUpdateHistory()
    } catch (error) {
      console.error('Load history failed:', error)
    }
  }

  async function clearHistory() {
    await api.clearUpdateHistory()
    updateHistory.value = []
  }

  return {
    isChecking,
    lastCheckedAt,
    appUpdate,
    pluginUpdates,
    engineUpdates,
    hotUpdate,
    currentHotUpdateVersion,
    updateHistory,
    isInstallingApp,
    isUpdatingPlugins,
    isInstallingHotUpdate,
    installProgress,
    installMessage,
    hotUpdateProgress,
    hotUpdateMessage,
    updateProgress,
    totalUpdateCount,
    hasAnyUpdate,
    initListeners,
    cleanupListeners,
    checkAll,
    installAppUpdate,
    skipAppVersion,
    updateSinglePlugin,
    batchUpdateAllPlugins,
    installHotUpdate,
    rollbackHotUpdate,
    loadHistory,
    clearHistory
  }
})
