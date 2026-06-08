import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api'
import type { AppUpdateInfo, PluginUpdateInfo, VersionUpdateInfo, HotUpdateInfo, UpdateHistoryEntry, UpdateProgress } from '@/types'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useToast } from '@/composables/useToast'

export const useUpdateStore = defineStore('updates', () => {
  const { t } = useI18n()
  const toast = useToast()
  const isChecking = ref(false)
  const hasChecked = ref(false)
  const checkError = ref('')
  const lastCheckedAt = ref('')
  const trayCheckMessage = ref('')
  const trayCheckHasUpdates = ref<boolean | null>(null)

  const appUpdate = ref<AppUpdateInfo | null>(null)
  const pluginUpdates = ref<PluginUpdateInfo[]>([])
  const engineUpdates = ref<VersionUpdateInfo[]>([])
  const hotUpdate = ref<HotUpdateInfo | null>(null)
  const currentHotUpdateVersion = ref<string | null>(null)
  const updateHistory = ref<UpdateHistoryEntry[]>([])

  const isInstallingApp = ref(false)
  const isUpdatingPlugins = ref(false)
  const updatingPluginId = ref<string | null>(null)
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
    if (hotUpdate.value) count++
    return count
  })

  const hasAnyUpdate = computed(() => totalUpdateCount.value > 0)

  const unlisteners: (() => void)[] = []

  async function initListeners() {
    if (unlisteners.length > 0) return
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
    const unlisten5 = await listen('tray-check-update-result', (event: any) => {
      trayCheckHasUpdates.value = event.payload.has_updates
      trayCheckMessage.value = event.payload.message
      if (!event.payload.has_updates) {
        setTimeout(() => {
          trayCheckHasUpdates.value = null
          trayCheckMessage.value = ''
        }, 5000)
      }
    })
    unlisteners.push(unlisten1, unlisten2, unlisten3, unlisten4, unlisten5)
  }

  function cleanupListeners() {
    unlisteners.forEach(fn => fn())
    unlisteners.length = 0
  }

  async function checkAll() {
    if (isChecking.value) return
    isChecking.value = true
    checkError.value = ''
    try {
      const result = await api.checkAllUpdates()
      appUpdate.value = result.app_update
      pluginUpdates.value = result.plugin_updates.filter(u => u.update_available)
      engineUpdates.value = result.engine_updates
      lastCheckedAt.value = result.checked_at

      if (!appUpdate.value) {
        try {
          const appUpd = await api.checkAppUpdate()
          appUpdate.value = appUpd
        } catch (e) {
          console.warn('App update check failed:', e)
        }
      }

      try {
        const hotUpd = await api.checkHotUpdate()
        hotUpdate.value = hotUpd
      } catch (e) {
        console.warn('Hot update check failed:', e)
      }

      try {
        currentHotUpdateVersion.value = await api.getCurrentHotUpdateVersion()
      } catch (e) {
        console.warn('Get current hot update version failed:', e)
      }

      try {
        updateHistory.value = await api.getUpdateHistory()
      } catch (e) {
        console.warn('Load update history failed:', e)
      }
    } catch (error) {
      console.error('Check updates failed:', error)
      checkError.value = String(error)
      toast.error(t('updates.checkFailed', { error: String(error) }))
    } finally {
      isChecking.value = false
      hasChecked.value = true
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

  async function reapplyBindingsForPlugin(pluginId: string) {
    try {
      const projects = await api.getProjects()
      const bindingResults = await Promise.allSettled(
        projects.map(p => api.getProjectBindings(p.project_id))
      )
      const projectIdsToApply: string[] = []
      bindingResults.forEach((result, i) => {
        if (result.status === 'fulfilled' && result.value.some(b => b.plugin_id === pluginId)) {
          projectIdsToApply.push(projects[i].project_id)
        }
      })
      await Promise.allSettled(
        projectIdsToApply.map(id => api.applyChanges(id))
      )
    } catch {
      // ignore reapply errors
    }
  }

  async function reapplyAllBindings() {
    try {
      const projects = await api.getProjects()
      await Promise.allSettled(
        projects.map(p => api.applyChanges(p.project_id))
      )
    } catch {
      // ignore reapply errors
    }
  }

  async function updateSinglePlugin(pluginId: string) {
    updatingPluginId.value = pluginId
    try {
      const updated = await api.updateGitPlugin(pluginId)
      pluginUpdates.value = pluginUpdates.value.filter(u => u.plugin_id !== pluginId)
      await reapplyBindingsForPlugin(pluginId)
      toast.success(t('plugins.updateSuccess', { name: updated.name }))
    } catch (error) {
      toast.error(t('plugins.updateFailed', { error: String(error) }))
    } finally {
      updatingPluginId.value = null
    }
  }

  async function batchUpdateAllPlugins() {
    isUpdatingPlugins.value = true
    try {
      const ids = pluginUpdates.value.map(u => u.plugin_id)
      const concurrency = 3
      const chunks: string[][] = []
      for (let i = 0; i < ids.length; i += concurrency) {
        chunks.push(ids.slice(i, i + concurrency))
      }
      const failedIds = new Set<string>()
      for (const chunk of chunks) {
        const results = await Promise.allSettled(
          chunk.map(id => api.updateGitPlugin(id))
        )
        for (let i = 0; i < results.length; i++) {
          if (results[i].status === 'rejected') {
            failedIds.add(chunk[i])
          }
        }
      }
      pluginUpdates.value = pluginUpdates.value.filter(u => failedIds.has(u.plugin_id))
      await reapplyAllBindings()
      if (failedIds.size === 0) {
        toast.success(t('plugins.batchUpdateSuccess'))
      } else {
        toast.error(t('plugins.batchUpdatePartial', { failed: failedIds.size }))
      }
    } catch (error) {
      toast.error(t('plugins.batchUpdateFailed', { error: String(error) }))
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
    hasChecked,
    checkError,
    lastCheckedAt,
    trayCheckMessage,
    trayCheckHasUpdates,
    appUpdate,
    pluginUpdates,
    engineUpdates,
    hotUpdate,
    currentHotUpdateVersion,
    updateHistory,
    isInstallingApp,
    isUpdatingPlugins,
    updatingPluginId,
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
