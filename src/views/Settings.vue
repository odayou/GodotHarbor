<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { Settings, LogEntry, TeamSharedConfig, Project } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '@/composables/useToast'
import { useTheme } from '@/composables/useTheme'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useOnboarding } from '@/composables/useOnboarding'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const { t, locale } = useI18n()
const { setTheme, initTheme } = useTheme()
const settings = ref<Settings>({ scan_directories: [], mount_strategy: 'Symlink', language: 'zh-CN', theme: 'system', auto_scan_on_startup: true, auto_discover_engines: true, plugin_storage_path: '', auto_check_plugin_updates: false, auto_check_app_updates: true, auto_check_engine_updates: true, update_check_interval_hours: 4, skipped_app_version: '' })
const isLoading = ref(false)
const logs = ref<LogEntry[]>([])
const showLogs = ref(false)
const showBackupDialog = ref(false)
const backupPath = ref('')
const isBackingUp = ref(false)
const isRestoring = ref(false)
const showTeamConfigDialog = ref(false)
const teamConfigs = ref<TeamSharedConfig[]>([])
const projects = ref<Project[]>([])
const showExportDialog = ref(false)
const exportConfigName = ref('')
const exportConfigDescription = ref('')
const selectedProjectIds = ref<string[]>([])
const isExporting = ref(false)
const isImporting = ref(false)

onMounted(() => { initTheme(); loadSettings(); loadTeamConfigs(); loadProjects() })

const loadSettings = async () => {
  isLoading.value = true
  try {
    const result = await api.getSettings()
    settings.value = { scan_directories: result.scan_directories || [], mount_strategy: result.mount_strategy || 'Symlink', language: result.language || 'zh-CN', theme: result.theme || 'system', auto_scan_on_startup: result.auto_scan_on_startup ?? true, auto_discover_engines: result.auto_discover_engines ?? true, plugin_storage_path: result.plugin_storage_path || '', auto_check_plugin_updates: result.auto_check_plugin_updates ?? false, auto_check_app_updates: result.auto_check_app_updates ?? true, auto_check_engine_updates: result.auto_check_engine_updates ?? true, update_check_interval_hours: result.update_check_interval_hours ?? 4, skipped_app_version: result.skipped_app_version || '' }
    oldPluginStoragePath.value = settings.value.plugin_storage_path || ''
    locale.value = settings.value.language
    if (['light', 'dark', 'system', 'volcano'].includes(settings.value.theme)) setTheme(settings.value.theme as 'light' | 'dark' | 'system' | 'volcano')
  } catch (error) { toast.error(t('settings.messages.loadFailed', { error })) }
  finally { isLoading.value = false }
}

watch(() => settings.value.language, (lang) => {
  locale.value = lang
  // 保存语言设置到localStorage
  localStorage.setItem('godotharbor-language', lang)
})
watch(() => settings.value.theme, (theme) => { if (['light', 'dark', 'system', 'volcano'].includes(theme)) setTheme(theme as 'light' | 'dark' | 'system' | 'volcano') })

const addScanDirectory = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('settings.scanDirs') })
    if (selected && typeof selected === 'string') {
      if (!settings.value.scan_directories) settings.value.scan_directories = []
      if (!settings.value.scan_directories.includes(selected)) { settings.value.scan_directories.push(selected); toast.info(t('settings.messages.addDir', { dir: selected })) }
      else toast.warning(t('settings.messages.dirExists'))
    }
  } catch (error) { toast.error(t('settings.messages.addDirFailed', { error })) }
}

const removeScanDirectory = (index: number) => { const dir = settings.value.scan_directories[index]; settings.value.scan_directories.splice(index, 1); toast.info(t('settings.messages.removeDir', { dir })) }

const saveSettings = async () => {
  isLoading.value = true
  try {
    await api.saveSettings(settings.value)
    toast.success(t('settings.messages.saveSuccess'))
  }
  catch (error) { toast.error(t('settings.messages.saveFailed', { error })) }
  finally { isLoading.value = false }
}

const saveSettingsWithMigrationCheck = async () => {
  if (settings.value.plugin_storage_path && oldPluginStoragePath.value &&
      settings.value.plugin_storage_path !== oldPluginStoragePath.value) {
    showMigrateDialog.value = true
    return
  }
  await saveSettings()
}

const migratePlugins = async () => {
  isMigrating.value = true
  try {
    await api.migratePluginStorage(oldPluginStoragePath.value, settings.value.plugin_storage_path || '')
    await saveSettings()
    toast.success(t('settings.pluginRepo.migrateSuccess'))
  } catch (error) {
    toast.error(t('settings.pluginRepo.migrateFailed', { error }))
  } finally {
    isMigrating.value = false
    showMigrateDialog.value = false
  }
}

const skipMigration = async () => {
  showMigrateDialog.value = false
  await saveSettings()
}

const loadLogs = async () => {
  try {
    logs.value = await api.getOperationLogs(50)
    showLogs.value = true
  } catch (error) { toast.error(t('settings.messages.loadLogsFailed', { error })) }
}

const copyError = async (log: LogEntry) => {
  try {
    await navigator.clipboard.writeText(log.detail)
    toast.success(t('settings.messages.copied'))
  } catch { toast.error(t('settings.messages.copyFailed')) }
}

const formatTime = (timestamp: string) => {
  try {
    const date = new Date(timestamp)
    return date.toLocaleString('zh-CN')
  } catch { return timestamp }
}

const selectBackupPath = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('settings.backup.selectDir') })
    if (selected && typeof selected === 'string') {
      backupPath.value = selected
    }
  } catch (error) { toast.error(t('settings.messages.selectDirFailed', { error })) }
}

const selectPluginStoragePath = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('settings.pluginRepo.storagePath') })
    if (selected && typeof selected === 'string') {
      settings.value.plugin_storage_path = selected
    }
  } catch (error) { toast.error(t('settings.messages.selectDirFailed', { error })) }
}

const oldPluginStoragePath = ref('')
const showMigrateDialog = ref(false)
const isMigrating = ref(false)

useDialogEscape(showMigrateDialog)

const performBackup = async () => {
  if (!backupPath.value) {
    toast.warning(t('settings.messages.selectDirFirst'))
    return
  }
  isBackingUp.value = true
  try {
    const result = await api.backupData(backupPath.value)
    toast.success(result)
    showBackupDialog.value = false
  } catch (error) {
    toast.error(t('settings.messages.backupFailed', { error }))
  } finally {
    isBackingUp.value = false
  }
}

const performRestore = async () => {
  if (!backupPath.value) {
    toast.warning(t('settings.messages.selectDirFirst'))
    return
  }
  isRestoring.value = true
  try {
    const result = await api.restoreData(backupPath.value)
    toast.success(result)
    await loadSettings()
    showBackupDialog.value = false
  } catch (error) {
    toast.error(t('settings.messages.restoreFailed', { error }))
  } finally {
    isRestoring.value = false
  }
}

const loadTeamConfigs = async () => {
  try {
    teamConfigs.value = await api.getTeamConfigs()
  } catch (error) {
    console.error('Failed to load team configs:', error)
  }
}

const loadProjects = async () => {
  try {
    projects.value = await api.getProjects()
  } catch (error) {
    console.error('Failed to load projects:', error)
  }
}

const openExportDialog = () => {
  exportConfigName.value = ''
  exportConfigDescription.value = ''
  selectedProjectIds.value = []
  showExportDialog.value = true
}

const exportTeamConfig = async () => {
  if (!exportConfigName.value) {
    toast.warning(t('settings.messages.enterConfigName'))
    return
  }
  if (selectedProjectIds.value.length === 0) {
    toast.warning(t('settings.messages.selectAtLeastOneProject'))
    return
  }
  isExporting.value = true
  try {
    await api.exportTeamConfig(exportConfigName.value, exportConfigDescription.value, selectedProjectIds.value)
    toast.success(t('settings.messages.exportSuccess'))
    showExportDialog.value = false
    await loadTeamConfigs()
  } catch (error) {
    toast.error(t('settings.messages.exportFailed', { error }))
  } finally {
    isExporting.value = false
  }
}

const importTeamConfig = async (configId: string) => {
  if (selectedProjectIds.value.length === 0) {
    toast.warning(t('settings.messages.selectAtLeastOneTarget'))
    return
  }
  isImporting.value = true
  try {
    await api.importTeamConfig(configId, selectedProjectIds.value)
    toast.success(t('settings.messages.importSuccess'))
    showTeamConfigDialog.value = false
    await loadTeamConfigs()
  } catch (error) {
    toast.error(t('settings.messages.importFailed', { error }))
  } finally {
    isImporting.value = false
  }
}

const showDeleteTeamConfigConfirm = ref(false)
const deleteTeamConfigId = ref('')
const showResetConfirm = ref(false)
const isResetting = ref(false)
const backupFingerprint = ref('')
const resetStep = ref(1)

useDialogEscape(showLogs)
useDialogEscape(showBackupDialog)
useDialogEscape(showTeamConfigDialog)
useDialogEscape(showExportDialog)
useDialogEscape(showResetConfirm)

const confirmResetData = () => {
  backupFingerprint.value = ''
  resetStep.value = 1
  showResetConfirm.value = true
}

const goToStep = (step: number) => {
  resetStep.value = step
}

const performReset = async () => {
  if (!backupFingerprint.value.trim()) {
    toast.warning(t('settings.messages.enterBackupFingerprint'))
    return
  }
  
  isResetting.value = true
  try {
    const result = await api.resetData(backupFingerprint.value.trim())
    toast.success(result)
    showResetConfirm.value = false
    backupFingerprint.value = ''
    resetStep.value = 1
  } catch (error) {
    toast.error(t('settings.messages.resetFailed', { error }))
  } finally {
    isResetting.value = false
  }
}

const confirmDeleteTeamConfig = (configId: string) => {
  deleteTeamConfigId.value = configId
  showDeleteTeamConfigConfirm.value = true
}

const onDeleteTeamConfigConfirm = async () => {
  try {
    await api.deleteTeamConfig(deleteTeamConfigId.value)
    toast.success(t('settings.messages.deleteSuccess'))
    await loadTeamConfigs()
  } catch (error) {
    toast.error(t('settings.messages.deleteFailed', { error }))
  }
}

const formatDate = (dateStr: string) => {
  try {
    return new Date(dateStr).toLocaleString('zh-CN')
  } catch {
    return dateStr
  }
}

const { showOnboarding } = useOnboarding()

const resetOnboarding = async () => {
  try {
    const currentSettings = await api.getSettings()
    currentSettings.onboarding_completed = false
    await api.saveSettings(currentSettings)
    showOnboarding()
  } catch (error) {
    toast.error(t('settings.messages.resetGuideFailed', { error }))
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('settings.title') }}</h1>
      <div class="flex gap-2">
        <button @click="loadLogs" class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-sm">{{ t('settings.buttons.viewLogs') }}</button>
        <button @click="showBackupDialog = true" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm">{{ t('settings.buttons.backup') }}</button>
        <button @click="showTeamConfigDialog = true" class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-sm">{{ t('settings.buttons.teamConfig') }}</button>
      </div>
    </div>
    <div v-if="isLoading" class="flex justify-center py-12"><div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div></div>
    <div v-else class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.scan') }}</h2>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.scanDirs') }}</label>
        <div class="space-y-2">
          <div v-for="(dir, index) in settings.scan_directories" :key="index" class="flex items-center space-x-2">
            <input type="text" readonly :value="dir" class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm" />
            <button @click="removeScanDirectory(index)" class="px-3 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm">{{ t('settings.remove') }}</button>
          </div>
          <div v-if="!settings.scan_directories?.length" class="text-sm text-gray-500 dark:text-gray-400 py-2">{{ t('settings.noDirs') }}</div>
          <button @click="addScanDirectory" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm">{{ t('settings.addDir') }}</button>
        </div>
        <div class="mt-4 space-y-3">
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="settings.auto_scan_on_startup" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.autoScanOnStartup') }}</span>
          </label>
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="settings.auto_discover_engines" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.autoDiscoverEngines') }}</span>
          </label>
        </div>
      </div>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.mount') }}</h2>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.mountStrategy') }}</label>
        <select v-model="settings.mount_strategy" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
          <option value="Symlink">{{ t('settings.symlink') }}</option>
          <option value="Junction">{{ t('settings.junction') }}</option>
          <option value="Copy">{{ t('settings.copy') }}</option>
        </select>
      </div>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.pluginRepo.title') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.pluginRepo.storagePath') }}</label>
            <div class="flex gap-2">
              <input
                type="text"
                v-model="settings.plugin_storage_path"
                :placeholder="t('settings.pluginRepo.placeholder')"
                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
              />
              <button
                @click="selectPluginStoragePath"
                class="px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300"
              >
                {{ t('settings.pluginRepo.browse') }}
              </button>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ t('settings.pluginRepo.storageHint') }}</p>
          </div>
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="settings.auto_check_plugin_updates" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.pluginRepo.autoCheckPluginUpdates') }}</span>
          </label>
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="settings.auto_check_app_updates" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.pluginRepo.autoCheckAppUpdates') }}</span>
          </label>
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="settings.auto_check_engine_updates" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.pluginRepo.autoCheckEngineUpdates') }}</span>
          </label>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.pluginRepo.checkInterval') }}</label>
            <input type="number" v-model.number="settings.update_check_interval_hours" min="1" max="168"
              class="w-32 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm" />
          </div>
        </div>
      </div>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.appearance') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.languageLabel') }}</label>
            <select v-model="settings.language" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
              <option value="zh-CN">{{ t('settings.language.zhCN') }}</option>
              <option value="en">English</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.theme') }}</label>
            <select v-model="settings.theme" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
              <option value="light">{{ t('settings.themeLight') }}</option>
              <option value="dark">{{ t('settings.themeDark') }}</option>
              <option value="system">{{ t('settings.themeSystem') }}</option>
              <option value="volcano">{{ t('settings.cloudProvider.volcano') }}</option>
            </select>
          </div>
        </div>
      </div>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.other') }}</h2>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.showOnboarding') }}</p>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{{ t('settings.showOnboardingDesc') }}</p>
            </div>
            <button
              @click="resetOnboarding"
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors text-sm"
            >
              {{ t('settings.showOnboarding') }}
            </button>
          </div>
          <div class="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-700">
            <div>
              <p class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.resetData') }}</p>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{{ t('settings.resetDataDesc') }}</p>
            </div>
            <button
              @click="confirmResetData"
              class="px-4 py-2 border border-red-300 dark:border-red-600 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-800/20 transition-colors text-sm"
            >
              {{ t('settings.resetData') }}
            </button>
          </div>
        </div>
      </div>
      <div class="flex justify-end">
        <button @click="saveSettingsWithMigrationCheck" :disabled="isLoading" class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50">{{ t('settings.save') }}</button>
      </div>
    </div>

    <div v-if="showLogs" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLogs = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-3xl shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('settings.logs.title') }}</h3>
          <button @click="showLogs = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
          </button>
        </div>
        <div class="flex-1 overflow-y-auto space-y-2">
          <div v-if="logs.length === 0" class="text-center py-8 text-gray-500 dark:text-gray-400">{{ t('settings.logs.empty') }}</div>
          <div v-for="(log, index) in logs" :key="index" :class="['p-3 rounded-lg border', log.level === 'error' ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800' : 'bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-600']">
            <div class="flex justify-between items-start">
              <div class="flex items-center gap-2">
                <span :class="['px-2 py-0.5 rounded text-xs font-medium', log.level === 'error' ? 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300' : 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300']">{{ log.level === 'error' ? t('settings.logs.error') : t('settings.logs.success') }}</span>
                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ log.action }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-500 dark:text-gray-400">{{ formatTime(log.timestamp) }}</span>
                <button v-if="log.level === 'error'" @click="copyError(log)" class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400">{{ t('settings.logs.copy') }}</button>
              </div>
            </div>
            <p v-if="log.target" class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ t('settings.logs.target', { target: log.target }) }}</p>
            <p :class="['text-sm mt-1', log.level === 'error' ? 'text-red-700 dark:text-red-300' : 'text-gray-600 dark:text-gray-400']">{{ log.detail }}</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showBackupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBackupDialog = false; backupPath = ''">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.backup.title') }}</h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
          {{ t('settings.backup.desc') }}
        </p>
        <div class="flex gap-2 mb-4">
          <input
            v-model="backupPath"
            type="text"
            readonly
            :placeholder="t('settings.backup.selectDir')"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          />
          <button
            @click="selectBackupPath"
            class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
          >
            {{ t('settings.backup.browse') }}
          </button>
        </div>
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
          <p class="text-xs text-yellow-800 dark:text-yellow-200">
            <strong>{{ t('settings.backup.warning') }}</strong>
          </p>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            @click="showBackupDialog = false; backupPath = ''"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('settings.backup.cancel') }}
          </button>
          <button
            @click="performBackup"
            :disabled="isBackingUp || !backupPath"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 transition-colors"
          >
            {{ isBackingUp ? t('settings.backup.restoring') : t('settings.backup.backup') }}
          </button>
          <button
            @click="performRestore"
            :disabled="isRestoring || !backupPath"
            class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
          >
            {{ isRestoring ? t('settings.backup.restoring') : t('settings.backup.restore') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showTeamConfigDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showTeamConfigDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-2xl shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('settings.teamConfig.title') }}</h3>
          <button @click="showTeamConfigDialog = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="mb-4">
          <button
            @click="openExportDialog"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm"
          >
            {{ t('settings.teamConfig.export') }}
          </button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div v-if="teamConfigs.length === 0" class="text-center py-8 text-gray-500 dark:text-gray-400">
            {{ t('settings.teamConfig.empty') }}
          </div>
          <div v-else class="space-y-4">
            <div v-for="config in teamConfigs" :key="config.config_id" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <div class="flex justify-between items-start">
                <div>
                  <h4 class="font-medium text-gray-900 dark:text-gray-100">{{ config.name }}</h4>
                  <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{{ config.description || t('settings.teamConfig.description') }}</p>
                  <p class="text-xs text-gray-400 dark:text-gray-500 mt-2">{{ t('settings.teamConfig.created', { date: formatDate(config.created_at) }) }}</p>
                </div>
                <div class="flex gap-2">
                  <button
                    @click="importTeamConfig(config.config_id)"
                    :disabled="isImporting || projects.length === 0"
                    class="px-3 py-1 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 transition-colors text-sm"
                  >
                    {{ t('settings.teamConfig.import') }}
                  </button>
                  <button
                    @click="confirmDeleteTeamConfig(config.config_id)"
                    class="px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700 text-sm"
                  >
                    {{ t('settings.teamConfig.delete') }}
                  </button>
                </div>
              </div>
              <div class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                {{ t('settings.teamConfig.stats', { bindings: config.bindings.length, engineBindings: config.engine_bindings.length }) }}
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-end mt-4">
          <button
            @click="showTeamConfigDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('settings.teamConfig.close') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showExportDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showExportDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.teamConfig.exportTitle') }}</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.teamConfig.name') }}</label>
            <input
              v-model="exportConfigName"
              type="text"
              :placeholder="t('settings.teamConfig.namePlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.teamConfig.descriptionLabel') }}</label>
            <input
              v-model="exportConfigDescription"
              type="text"
              :placeholder="t('settings.teamConfig.descriptionPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.teamConfig.selectProjects') }}</label>
            <div class="space-y-2 max-h-40 overflow-y-auto bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
              <div v-for="project in projects" :key="project.project_id" class="flex items-center gap-2">
                <input
                  type="checkbox"
                  :value="project.project_id"
                  v-model="selectedProjectIds"
                  class="w-4 h-4 text-primary-600 rounded"
                />
                <span class="text-sm text-gray-900 dark:text-gray-100">{{ project.name }}</span>
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showExportDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('settings.teamConfig.cancel') }}
          </button>
          <button
            @click="exportTeamConfig"
            :disabled="isExporting || !exportConfigName || selectedProjectIds.length === 0"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ isExporting ? t('settings.teamConfig.exporting') : t('settings.teamConfig.exportAction') }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteTeamConfigConfirm"
      :title="t('settings.teamConfig.deleteConfirmTitle')"
      :description="t('settings.teamConfig.deleteConfirmDesc')"
      :confirm-text="t('settings.teamConfig.delete')"
      @confirm="onDeleteTeamConfigConfirm"
    />

    <div v-if="showMigrateDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showMigrateDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.pluginRepo.migrateTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-secondary mb-3">
          {{ t('settings.pluginRepo.migrateDesc') }}
        </p>
        <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 mb-4 text-xs font-mono space-y-1">
          <div class="text-red-500 dark:text-red-400">{{ t('settings.pluginRepo.migrateFrom') }}: {{ oldPluginStoragePath }}</div>
          <div class="text-green-500 dark:text-green-400">{{ t('settings.pluginRepo.migrateTo') }}: {{ settings.plugin_storage_path }}</div>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="skipMigration" :disabled="isMigrating" class="btn-secondary">{{ t('settings.pluginRepo.skipMigration') }}</button>
          <button @click="migratePlugins" :disabled="isMigrating" class="btn-primary disabled:opacity-50">
            {{ isMigrating ? t('settings.pluginRepo.migrating') : t('settings.pluginRepo.startMigration') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showResetConfirm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showResetConfirm = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <div class="flex items-center gap-3 mb-6">
          <div class="w-10 h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
            <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('settings.resetDataConfirmTitle') }}</h3>
        </div>

        <div class="flex items-center justify-center gap-2 mb-6">
          <div :class="['w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold', resetStep >= 1 ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400']">1</div>
          <div :class="['flex-1 h-1', resetStep >= 2 ? 'bg-primary-600' : 'bg-gray-200 dark:bg-gray-700']"></div>
          <div :class="['w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold', resetStep >= 2 ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400']">2</div>
          <div :class="['flex-1 h-1', resetStep >= 3 ? 'bg-primary-600' : 'bg-gray-200 dark:bg-gray-700']"></div>
          <div :class="['w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold', resetStep >= 3 ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400']">3</div>
        </div>

        <div v-if="resetStep === 1" class="mb-6">
          <p class="text-sm text-gray-600 dark:text-content-secondary mb-4">
            {{ t('settings.resetDataStep1Desc') }}
          </p>
          <ul class="text-sm text-gray-500 dark:text-content-secondary space-y-2 mb-4 bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            <li>{{ t('settings.resetDataItem.projects') }}</li>
            <li>{{ t('settings.resetDataItem.plugins') }}</li>
            <li>{{ t('settings.resetDataItem.engines') }}</li>
            <li>{{ t('settings.resetDataItem.bindings') }}</li>
            <li>{{ t('settings.resetDataItem.settings') }}</li>
          </ul>
          <button @click="goToStep(2)" class="w-full btn-primary">
            {{ t('settings.resetDataStep1Continue') }}
          </button>
        </div>

        <div v-if="resetStep === 2" class="mb-6">
          <p class="text-sm text-gray-600 dark:text-content-secondary mb-4">
            {{ t('settings.resetDataStep2Desc') }}
          </p>
          <div class="bg-yellow-50 dark:bg-yellow-900/20 rounded-lg p-3 mb-4">
            <p class="text-sm text-yellow-800 dark:text-yellow-300">
              {{ t('settings.resetDataStep2Hint') }}
            </p>
          </div>
          <input
            v-model="backupFingerprint"
            type="text"
            :placeholder="t('settings.resetDataStep2Placeholder')"
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400"
          />
          <div class="flex justify-end gap-3 mt-4">
            <button @click="goToStep(1)" class="btn-secondary">{{ t('common.back') }}</button>
            <button @click="goToStep(3)" :disabled="!backupFingerprint.trim()" class="btn-primary disabled:opacity-50">
              {{ t('common.next') }}
            </button>
          </div>
        </div>

        <div v-if="resetStep === 3" class="mb-6">
          <p class="text-sm text-red-600 dark:text-red-400 mb-4">
            {{ t('settings.resetDataStep3Desc') }}
          </p>
          <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 mb-4">
            <p class="text-sm text-gray-600 dark:text-content-secondary">
              {{ t('settings.resetDataStep3BackupPath') }}
            </p>
            <p class="text-sm font-mono text-gray-800 dark:text-gray-200 mt-1 break-all">
              {{ backupFingerprint }}
            </p>
          </div>
          <div class="flex justify-end gap-3">
            <button @click="goToStep(2)" class="btn-secondary">{{ t('common.back') }}</button>
            <button @click="performReset" :disabled="isResetting" class="btn-primary disabled:opacity-50">
              {{ isResetting ? t('settings.resetting') : t('settings.confirmReset') }}
            </button>
          </div>
        </div>

        <button @click="showResetConfirm = false" class="w-full mt-4 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">
          {{ t('common.cancel') }}
        </button>
      </div>
    </div>
  </div>
</template>