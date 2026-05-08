<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { onBeforeRouteLeave } from 'vue-router'
import { api } from '@/api'
import type { Settings, LogEntry, Project, EngineMirrorConfig, StoragePaths } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '@/composables/useToast'
import { useTheme } from '@/composables/useTheme'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useOnboarding } from '@/composables/useOnboarding'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const { t, locale } = useI18n()
const { setTheme, initTheme } = useTheme()
const settings = ref<Settings>({ scan_directories: [], mount_strategy: 'Symlink', language: 'zh-CN', theme: 'system', auto_scan_on_startup: true, auto_discover_engines: true, auto_check_plugin_updates: false, auto_check_app_updates: true, auto_check_engine_updates: true, update_check_interval_hours: 4, skipped_app_version: '', auto_apply: false, github_api_proxy: '', asset_library_mirror: '' })
const originalSettings = ref<string>('')
const isLoading = ref(false)
const isDirty = computed(() => {
  return JSON.stringify(settings.value) !== originalSettings.value
})
const showUnsavedDialog = ref(false)
let pendingNavigation: (() => void) | null = null

watch(() => settings.value.theme, async (newTheme) => {
  try {
    const current = await api.getSettings()
    current.theme = newTheme
    await api.saveSettings(current)
    originalSettings.value = JSON.stringify(settings.value)
  } catch {}
})

watch(() => settings.value.language, async (newLang) => {
  try {
    const current = await api.getSettings()
    current.language = newLang
    await api.saveSettings(current)
    originalSettings.value = JSON.stringify(settings.value)
  } catch {}
})

const logs = ref<LogEntry[]>([])
const showLogs = ref(false)
const logSortOrder = ref<'newest' | 'oldest'>('newest')

const sortedLogs = computed(() => {
  const sorted = [...logs.value]
  return logSortOrder.value === 'newest' ? sorted : sorted.reverse()
})
const showBackupDialog = ref(false)
const showRestoreDialog = ref(false)
const backupPath = ref('')
const isBackingUp = ref(false)
const isRestoring = ref(false)
const projects = ref<Project[]>([])

const activeSection = ref('general')

onMounted(() => {
  initTheme(); loadSettings(); loadProjects(); loadStoragePaths()
})

onBeforeRouteLeave((_to, _from, next) => {
  if (isDirty.value) {
    pendingNavigation = () => next(true)
    showUnsavedDialog.value = true
    next(false)
  } else {
    next(true)
  }
})

const discardChanges = () => {
  showUnsavedDialog.value = false
  if (pendingNavigation) {
    pendingNavigation()
    pendingNavigation = null
  }
}

const saveAndLeave = async () => {
  await saveSettings()
  showUnsavedDialog.value = false
  if (pendingNavigation) {
    pendingNavigation()
    pendingNavigation = null
  }
}

const loadSettings = async () => {
  isLoading.value = true
  try {
    const result = await api.getSettings()
    settings.value = { scan_directories: result.scan_directories || [], mount_strategy: result.mount_strategy || 'Symlink', language: result.language || 'zh-CN', theme: result.theme || 'system', auto_scan_on_startup: result.auto_scan_on_startup ?? true, auto_discover_engines: result.auto_discover_engines ?? true, auto_check_plugin_updates: result.auto_check_plugin_updates ?? false, auto_check_app_updates: result.auto_check_app_updates ?? true, auto_check_engine_updates: result.auto_check_engine_updates ?? true, update_check_interval_hours: result.update_check_interval_hours ?? 4, skipped_app_version: result.skipped_app_version || '', auto_apply: result.auto_apply ?? false, github_api_proxy: result.github_api_proxy || '', asset_library_mirror: result.asset_library_mirror || '' }
    const localStorageLang = localStorage.getItem('godotharbor-language')
    if (localStorageLang && localStorageLang !== settings.value.language) {
      settings.value.language = localStorageLang
    }
    locale.value = settings.value.language
    if (['light', 'dark', 'system', 'volcano'].includes(settings.value.theme)) setTheme(settings.value.theme as 'light' | 'dark' | 'system' | 'volcano')
  } catch (error) { toast.error(t('settings.messages.loadFailed', { error })) }
  finally { isLoading.value = false; originalSettings.value = JSON.stringify(settings.value) }
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
    await loadStoragePaths()
    originalSettings.value = JSON.stringify(settings.value)
    toast.success(t('settings.messages.saveSuccess'))
  }
  catch (error) { toast.error(t('settings.messages.saveFailed', { error })) }
  finally { isLoading.value = false }
}

const saveSettingsWithMigrationCheck = async () => {
  if (checkDataDirChange()) {
    return
  }
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
    return date.toLocaleString(settings.value.language || 'zh-CN')
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

const loadStoragePaths = async () => {
  try {
    storagePaths.value = await api.getStoragePaths()
  } catch (error) {
    console.error('Failed to load storage paths:', error)
  }
}

const openPath = async (path: string) => {
  try {
    await api.openInFileManager(path)
  } catch (error) {
    toast.error(t('settings.messages.selectDirFailed', { error }))
  }
}

const selectCustomDataDir = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('settings.storage.customDataDir') })
    if (selected && typeof selected === 'string') {
      settings.value.custom_data_dir = selected
    }
  } catch (error) { toast.error(t('settings.messages.selectDirFailed', { error })) }
}

const showDataMigrateDialog = ref(false)
const isMigratingData = ref(false)
const pendingDataDir = ref('')

const checkDataDirChange = () => {
  const oldDir = storagePaths.value?.app_data_dir || ''
  const newDir = settings.value.custom_data_dir || ''
  if (oldDir !== newDir && newDir) {
    pendingDataDir.value = newDir
    showDataMigrateDialog.value = true
    return true
  }
  return false
}

const executeDataMigration = async () => {
  isMigratingData.value = true
  try {
    await api.migrateDataDir(pendingDataDir.value)
    toast.success(t('settings.storage.migrateSuccess'))
    showDataMigrateDialog.value = false
    await loadSettings()
    await loadStoragePaths()
  } catch (error) {
    toast.error(t('settings.storage.migrateFailed') + ': ' + error)
  } finally {
    isMigratingData.value = false
  }
}

const storagePaths = ref<StoragePaths | null>(null)

useDialogEscape(showDataMigrateDialog)

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
  } catch (error: any) {
    const msg = String(error)
    if (msg.includes('invalid') || msg.includes('corrupt') || msg.includes('not found') || msg.includes('损坏') || msg.includes('无效')) {
      toast.error(t('settings.messages.invalidBackupFile'))
    } else {
      toast.error(t('settings.messages.restoreFailed', { error }))
    }
  } finally {
    isRestoring.value = false
  }
}

const loadProjects = async () => {
  try {
    projects.value = await api.getProjects()
  } catch (error) {
    console.error('Failed to load projects:', error)
  }
}

const showRestoreConfirm = ref(false)
const showResetConfirm = ref(false)
const isResetting = ref(false)
const backupFingerprint = ref('')
const resetStep = ref(1)

useDialogEscape(showLogs)
useDialogEscape(showBackupDialog)
useDialogEscape(showResetConfirm)

const confirmResetData = () => {
  backupFingerprint.value = ''
  resetStep.value = 1
  showResetConfirm.value = true
}

const goToStep = (step: number) => {
  resetStep.value = step
}

const selectResetBackupPath = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('settings.resetData.selectBackup') })
    if (selected && typeof selected === 'string') {
      backupFingerprint.value = selected
    }
  } catch (error) { toast.error(t('settings.messages.selectDirFailed', { error })) }
}

const performReset = async () => {
  if (!backupFingerprint.value.trim()) {
    toast.warning(t('settings.messages.selectDirFirst'))
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

const showMirrorDialog = ref(false)
const editingMirror = ref<EngineMirrorConfig | null>(null)
const mirrorFormName = ref('')
const mirrorFormUrl = ref('')
const mirrorFormEnabled = ref(true)
const mirrorFormType = ref('github_api')

useDialogEscape(showMirrorDialog)

const openAddMirror = () => {
  editingMirror.value = null
  mirrorFormName.value = ''
  mirrorFormUrl.value = ''
  mirrorFormEnabled.value = true
  mirrorFormType.value = 'github_api'
  showMirrorDialog.value = true
}

const openEditMirror = (mirror: EngineMirrorConfig) => {
  editingMirror.value = mirror
  mirrorFormName.value = mirror.name
  mirrorFormUrl.value = mirror.base_url
  mirrorFormEnabled.value = mirror.enabled
  mirrorFormType.value = mirror.mirror_type || 'github_api'
  showMirrorDialog.value = true
}

const saveMirror = () => {
  if (!mirrorFormName.value.trim() || !mirrorFormUrl.value.trim()) {
    toast.warning(t('settings.engineMirror.nameUrlRequired'))
    return
  }

  if (!settings.value.engine_mirrors) {
    settings.value.engine_mirrors = []
  }

  if (editingMirror.value) {
    const mirror = settings.value.engine_mirrors.find(m => m.id === editingMirror.value!.id)
    if (mirror) {
      mirror.name = mirrorFormName.value.trim()
      mirror.base_url = mirrorFormUrl.value.trim()
      mirror.enabled = mirrorFormEnabled.value
      mirror.mirror_type = mirrorFormType.value
    }
  } else {
    const newMirror: EngineMirrorConfig = {
      id: `mirror_${Date.now()}`,
      name: mirrorFormName.value.trim(),
      base_url: mirrorFormUrl.value.trim(),
      enabled: mirrorFormEnabled.value,
      is_official: false,
      mirror_type: mirrorFormType.value,
    }
    settings.value.engine_mirrors.push(newMirror)
  }

  showMirrorDialog.value = false
  toast.info(t('settings.engineMirror.saveHint'))
}

const removeMirror = (mirrorId: string) => {
  if (!settings.value.engine_mirrors) return
  const mirror = settings.value.engine_mirrors.find(m => m.id === mirrorId)
  if (mirror?.is_official) return
  settings.value.engine_mirrors = settings.value.engine_mirrors.filter(m => m.id !== mirrorId)
  toast.info(t('settings.engineMirror.saveHint'))
}

const toggleMirrorEnabled = (mirrorId: string) => {
  if (!settings.value.engine_mirrors) return
  const mirror = settings.value.engine_mirrors.find(m => m.id === mirrorId)
  if (mirror) {
    mirror.enabled = !mirror.enabled
  }
}
</script>

<template>
  <div class="relative">
    <div class="space-y-6">
      <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">{{ t('settings.title') }}</h1>
      <div class="flex gap-2">
        <button @click="loadLogs" class="px-4 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm">{{ t('settings.buttons.viewLogs') }}</button>
      </div>
    </div>
    <div v-if="isLoading" class="flex justify-center py-12"><div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div></div>
    <div v-else class="flex gap-6 items-start">
      <nav class="w-44 shrink-0 hidden lg:block">
        <div class="sticky top-6 space-y-1">
          <button v-for="section in [
            { id: 'general', label: t('settings.general'), icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' },
            { id: 'data', label: t('settings.data'), icon: 'M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4' },
            { id: 'mount', label: t('settings.mount'), icon: 'M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1' },
            { id: 'updates', label: t('settings.updates.title'), icon: 'M13 10V3L4 14h7v7l9-11h-7z' }
          ]" :key="section.id" @click="activeSection = section.id"
            :class="[
              'flex items-center gap-2.5 px-3 py-2 text-sm rounded-lg transition-colors w-full text-left',
              activeSection === section.id
                ? 'bg-primary-50 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 font-medium'
                : 'text-gray-600 dark:text-content-muted hover:bg-primary-50 dark:hover:bg-primary-900/20 hover:text-primary-600 dark:hover:text-primary-400'
            ]"
          >
            <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="section.icon" />
            </svg>
            {{ section.label }}
          </button>
        </div>
      </nav>
      <div class="lg:hidden flex gap-1.5 overflow-x-auto pb-2 -mx-1 px-1">
        <button v-for="section in [
          { id: 'general', label: t('settings.general') },
          { id: 'data', label: t('settings.data') },
          { id: 'mount', label: t('settings.mount') },
          { id: 'updates', label: t('settings.updates.title') }
        ]" :key="section.id" @click="activeSection = section.id"
          :class="[
            'px-3 py-1.5 text-xs rounded-full whitespace-nowrap transition-colors',
            activeSection === section.id
              ? 'bg-primary-600 text-white font-medium'
              : 'bg-gray-100 dark:bg-surface-hover text-gray-600 dark:text-content-muted'
          ]"
        >
          {{ section.label }}
        </button>
      </div>
      <div class="flex-1 min-w-0 space-y-6">
      <div v-show="activeSection === 'general'" class="space-y-6">
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.appearance') }}</h2>
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.languageLabel') }}</label>
              <select v-model="settings.language" class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary">
                <option value="zh-CN">{{ t('settings.language.zhCN') }}</option>
                <option value="en">English</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.theme') }}</label>
              <select v-model="settings.theme" class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary">
                <option value="light">{{ t('settings.themeLight') }}</option>
                <option value="dark">{{ t('settings.themeDark') }}</option>
                <option value="system">{{ t('settings.themeSystem') }}</option>
                <option value="volcano">{{ t('settings.cloudProvider.volcano') }}</option>
              </select>
            </div>
          </div>
        </div>
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.scan') }}</h2>
          <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.scanDirs') }}</label>
          <div class="space-y-2">
            <div v-for="(dir, index) in settings.scan_directories" :key="index" class="flex items-center space-x-2">
              <input type="text" readonly :value="dir" class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-gray-50 dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm" />
              <button @click="removeScanDirectory(index)" class="px-3 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm">{{ t('settings.remove') }}</button>
            </div>
            <div v-if="!settings.scan_directories?.length" class="text-sm text-gray-500 dark:text-content-muted py-2">{{ t('settings.noDirs') }}</div>
            <button @click="addScanDirectory" class="btn-primary text-sm">{{ t('settings.addDir') }}</button>
          </div>
          <div class="mt-4 space-y-3">
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_scan_on_startup" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.autoScanOnStartup') }}</span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_discover_engines" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.autoDiscoverEngines') }}</span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_apply" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.autoApply') }}</span>
            </label>
            <p v-if="settings.auto_apply" class="text-xs text-gray-500 dark:text-content-muted ml-7">{{ t('settings.autoApplyDesc') }}</p>
          </div>
        </div>
      </div>
      <div v-show="activeSection === 'mount'" class="card p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.mount') }}</h2>
        <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.mountStrategy') }}</label>
        <select v-model="settings.mount_strategy" class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary">
          <option value="Symlink">{{ t('settings.symlink') }}</option>
          <option value="Junction">{{ t('settings.junction') }}</option>
          <option value="Copy">{{ t('settings.copy') }}</option>
        </select>
        <p class="text-xs text-gray-500 dark:text-content-muted mt-1.5">
          <span v-if="settings.mount_strategy === 'Symlink'">{{ t('settings.symlinkDesc') }}</span>
          <span v-else-if="settings.mount_strategy === 'Junction'">{{ t('settings.junctionDesc') }}</span>
          <span v-else-if="settings.mount_strategy === 'Copy'">{{ t('settings.copyDesc') }}</span>
        </p>
      </div>
      <div v-if="storagePaths" v-show="activeSection === 'data'" class="space-y-6">
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('settings.storage.title') }}</h2>
          <p class="text-xs text-gray-500 dark:text-content-muted mb-4">{{ t('settings.storage.pathHierarchy') }}</p>
          <div class="mb-4 p-3 border border-gray-200 dark:border-surface-border rounded-lg">
            <label class="block text-sm font-medium text-gray-900 dark:text-content-primary mb-1">{{ t('settings.storage.customDataDir') }}</label>
            <p class="text-xs text-gray-500 dark:text-content-muted mb-2">{{ t('settings.storage.customDataDirDesc') }}</p>
            <div class="flex gap-2">
              <input type="text" v-model="settings.custom_data_dir"
                     :placeholder="t('settings.storage.customDataDirPlaceholder')"
                     class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent" />
              <button @click="selectCustomDataDir" class="px-3 py-2 text-sm border border-gray-300 dark:border-surface-border rounded-lg hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary transition-colors">{{ t('settings.pluginRepo.browse') }}</button>
              <button v-if="settings.custom_data_dir" @click="settings.custom_data_dir = ''" class="px-3 py-2 text-sm border border-gray-300 dark:border-surface-border rounded-lg hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary transition-colors">{{ t('settings.storage.resetToDefault') }}</button>
            </div>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-2">{{ t('settings.storage.customDataDirHint') }}</p>
          </div>
          <div class="text-xs text-gray-500 dark:text-content-muted mb-2">{{ t('settings.storage.currentPaths') }}</div>
          <div class="border border-gray-200 dark:border-surface-border rounded-lg overflow-hidden">
            <table class="w-full text-xs">
              <thead class="bg-gray-50 dark:bg-surface-hover/50">
                <tr>
                  <th class="text-left px-3 py-1.5 font-medium text-gray-500 dark:text-content-muted w-24">{{ t('settings.storage.pathName') }}</th>
                  <th class="text-left px-3 py-1.5 font-medium text-gray-500 dark:text-content-muted">{{ t('settings.storage.pathValue') }}</th>
                  <th class="text-right px-3 py-1.5 font-medium text-gray-500 dark:text-content-muted w-16"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                <tr v-for="item in [
                  { key: 'appDataDir', path: storagePaths.app_data_dir },
                  { key: 'pluginsDir', path: storagePaths.plugins_dir },
                  { key: 'enginesDir', path: storagePaths.engines_dir },
                  { key: 'cacheDir', path: storagePaths.cache_dir },
                  { key: 'logsDir', path: storagePaths.logs_dir },
                  { key: 'hotUpdatesDir', path: storagePaths.hot_updates_dir },
                  { key: 'settingsFile', path: storagePaths.settings_file },
                  { key: 'projectsFile', path: storagePaths.projects_file },
                  { key: 'enginesFile', path: storagePaths.engines_file }
                ]" :key="item.key">
                  <td class="px-3 py-1.5 text-gray-700 dark:text-content-secondary whitespace-nowrap">{{ t(`settings.storage.${item.key}`) }}</td>
                  <td class="px-3 py-1.5 font-mono text-gray-600 dark:text-content-muted break-all">{{ item.path }}</td>
                  <td class="px-3 py-1.5 text-right"><button @click="openPath(item.path)" class="text-primary-600 dark:text-primary-400 hover:underline">{{ t('settings.storage.open') }}</button></td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.dataOps') }}</h2>
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.buttons.backup') }}</p>
                <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">{{ t('settings.backup.desc') }}</p>
              </div>
              <div class="flex gap-2">
                <button @click="showBackupDialog = true" class="px-4 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors text-sm">{{ t('settings.buttons.backup') }}</button>
                <button @click="showRestoreDialog = true" class="px-4 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors text-sm">{{ t('settings.backup.restore') }}</button>
              </div>
            </div>
            <div class="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-surface-border">
              <div>
                <p class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.showOnboarding') }}</p>
                <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">{{ t('settings.showOnboardingDesc') }}</p>
              </div>
              <button @click="resetOnboarding" class="px-4 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors text-sm">{{ t('settings.showOnboarding') }}</button>
            </div>
            <div class="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-surface-border">
              <div>
                <p class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.resetDataLabel') }}</p>
                <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">{{ t('settings.resetDataDesc') }}</p>
              </div>
              <button @click="confirmResetData" class="px-4 py-2 border border-red-300 dark:border-red-600 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-800/20 transition-colors text-sm">{{ t('settings.resetDataLabel') }}</button>
            </div>
          </div>
        </div>
      </div>
      <div v-show="activeSection === 'updates'" class="space-y-6">
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.updates.autoCheck') }}</h2>
          <div class="space-y-3">
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_check_app_updates" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.pluginRepo.autoCheckAppUpdates') }}</span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_check_plugin_updates" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.pluginRepo.autoCheckPluginUpdates') }}</span>
            </label>
            <label class="flex items-center gap-3 cursor-pointer">
              <input type="checkbox" v-model="settings.auto_check_engine_updates" class="w-4 h-4 text-primary-600 rounded" />
              <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.pluginRepo.autoCheckEngineUpdates') }}</span>
            </label>
            <div class="pt-2">
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.pluginRepo.checkInterval') }}</label>
              <input type="number" v-model.number="settings.update_check_interval_hours" min="1" max="168"
                class="w-32 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm" />
            </div>
          </div>
        </div>
        <div class="card p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.engineMirror.title') }}</h2>
          <p class="text-sm text-gray-500 dark:text-content-muted mb-4">{{ t('settings.engineMirror.desc') }}</p>

          <div class="mb-5 p-4 rounded-lg border border-gray-200 dark:border-surface-border">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.networkProxy.githubApiProxy') }}</label>
            <input
              v-model="settings.github_api_proxy"
              type="text"
              :placeholder="t('settings.networkProxy.githubApiProxyPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            />
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.networkProxy.githubApiProxyHint') }}</p>
          </div>

          <div class="mb-5 p-4 rounded-lg border border-gray-200 dark:border-surface-border">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.networkProxy.assetLibraryMirror') }}</label>
            <input
              v-model="settings.asset_library_mirror"
              type="text"
              :placeholder="t('settings.networkProxy.assetLibraryMirrorPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            />
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.networkProxy.assetLibraryMirrorHint') }}</p>
          </div>

          <div class="space-y-3">
            <div v-for="mirror in (settings.engine_mirrors || [])" :key="mirror.id"
              class="flex items-center gap-3 p-3 rounded-lg border border-gray-200 dark:border-surface-border"
              :class="{ 'opacity-60': !mirror.enabled }"
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ mirror.name }}</span>
                  <span v-if="mirror.is_official" class="px-1.5 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">{{ t('settings.engineMirror.official') }}</span>
                  <span v-else class="px-1.5 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-surface-hover dark:text-content-muted">{{ t('settings.engineMirror.custom') }}</span>
                </div>
                <span class="text-xs text-gray-500 dark:text-content-muted truncate block mt-0.5">{{ mirror.base_url }}</span>
              </div>
              <div class="flex items-center gap-2">
                <button
                  @click="toggleMirrorEnabled(mirror.id)"
                  :class="['px-2 py-1 rounded text-xs font-medium transition-colors', mirror.enabled ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-500 dark:bg-surface-hover dark:text-content-muted']"
                >
                  {{ mirror.enabled ? t('settings.engineMirror.enabled') : t('settings.engineMirror.disabled') }}
                </button>
                <button
                  @click="openEditMirror(mirror)"
                  class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-1 rounded hover:bg-gray-100 dark:hover:bg-surface-layer transition-colors"
                  :title="t('settings.engineMirror.edit')"
                >
                  <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
                </button>
                <button
                  v-if="!mirror.is_official"
                  @click="removeMirror(mirror.id)"
                  class="text-red-500 hover:text-red-700 p-1 rounded hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                  :title="t('settings.engineMirror.remove')"
                >
                  <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                </button>
              </div>
            </div>
            <button
              @click="openAddMirror"
              class="px-4 py-2 border border-dashed border-gray-300 dark:border-surface-border text-gray-600 dark:text-content-muted rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm w-full"
            >
              + {{ t('settings.engineMirror.addMirror') }}
            </button>
          </div>
        </div>
      </div>
      </div>
    </div>
    </div>

    <Transition
      enter-active-class="transition-all duration-300"
      enter-from-class="translate-y-full opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition-all duration-200"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-full opacity-0"
    >
      <div v-if="isDirty" class="fixed bottom-0 left-0 right-0 bg-white dark:bg-surface-card border-t border-primary-200 dark:border-primary-800 shadow-lg z-40 px-6 py-3 flex items-center justify-between">
        <p class="text-sm text-gray-600 dark:text-content-muted">{{ t('settings.unsavedChanges') }}</p>
        <div class="flex gap-3">
          <button @click="loadSettings" class="px-4 py-2 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm">{{ t('settings.discardChanges') }}</button>
          <button @click="saveSettingsWithMigrationCheck" :disabled="isLoading" class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm">{{ t('settings.save') }}</button>
        </div>
      </div>
    </Transition>
  </div>

  <Teleport to="body">
  <div v-if="showLogs" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLogs = false">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-3xl shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('settings.logs.title') }}</h3>
          <div class="flex items-center gap-3">
            <button
              @click="logSortOrder = logSortOrder === 'newest' ? 'oldest' : 'newest'"
              class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 flex items-center gap-1"
            >
              <svg class="w-3.5 h-3.5" :class="{ 'rotate-180': logSortOrder === 'oldest' }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
              {{ logSortOrder === 'newest' ? t('settings.logs.sortNewest') : t('settings.logs.sortOldest') }}
            </button>
            <button @click="showLogs = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
              <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            </button>
          </div>
        </div>
        <div class="flex-1 overflow-y-auto space-y-2">
          <div v-if="sortedLogs.length === 0" class="text-center py-8 text-gray-500 dark:text-content-muted">{{ t('settings.logs.empty') }}</div>
          <div v-for="(log, index) in sortedLogs" :key="index" :class="['p-3 rounded-lg border', log.level === 'error' ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800' : 'bg-gray-50 dark:bg-surface-hover border-gray-200 dark:border-surface-border']">
            <div class="flex justify-between items-start">
              <div class="flex items-center gap-2">
                <span :class="['px-2 py-0.5 rounded text-xs font-medium', log.level === 'error' ? 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300' : 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300']">{{ log.level === 'error' ? t('settings.logs.error') : t('settings.logs.success') }}</span>
                <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ log.action }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-500 dark:text-content-muted">{{ formatTime(log.timestamp) }}</span>
                <button v-if="log.level === 'error'" @click="copyError(log)" class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400">{{ t('settings.logs.copy') }}</button>
              </div>
            </div>
            <p v-if="log.target" class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.logs.target', { target: log.target }) }}</p>
            <p :class="['text-sm mt-1', log.level === 'error' ? 'text-red-700 dark:text-red-300' : 'text-gray-600 dark:text-content-muted']">{{ log.detail }}</p>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showBackupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBackupDialog = false; backupPath = ''">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.backup.title') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-4">
          {{ t('settings.backup.desc') }}
        </p>
        <div class="flex gap-2 mb-4">
          <input
            v-model="backupPath"
            type="text"
            readonly
            :placeholder="t('settings.backup.selectDir')"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-gray-50 dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
          />
          <button
            @click="selectBackupPath"
            class="btn-secondary text-sm whitespace-nowrap"
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
            class="btn-secondary"
          >
            {{ t('settings.backup.cancel') }}
          </button>
          <button
            @click="performBackup"
            :disabled="isBackingUp || !backupPath"
            class="btn-primary disabled:opacity-50"
          >
            {{ isBackingUp ? t('settings.backup.backupping') : t('settings.backup.backup') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showRestoreDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRestoreDialog = false; backupPath = ''">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.backup.restoreTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-4">{{ t('settings.backup.restoreDesc') }}</p>
        <div class="flex gap-2 mb-4">
          <input v-model="backupPath" type="text" readonly :placeholder="t('settings.backup.selectDir')" class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-gray-50 dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm" />
          <button @click="selectBackupPath" class="btn-secondary text-sm whitespace-nowrap">{{ t('settings.backup.browse') }}</button>
        </div>
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
          <p class="text-xs text-yellow-800 dark:text-yellow-200"><strong>{{ t('settings.backup.restoreWarning') }}</strong></p>
        </div>
        <div class="flex justify-end space-x-3">
          <button @click="showRestoreDialog = false; backupPath = ''" class="btn-secondary">{{ t('settings.backup.cancel') }}</button>
          <button @click="showRestoreConfirm = true" :disabled="isRestoring || !backupPath" class="btn-primary disabled:opacity-50">{{ isRestoring ? t('settings.backup.restoring') : t('settings.backup.restore') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <ConfirmDialog
      v-model="showRestoreConfirm"
      :title="t('settings.storage.backup.restoreConfirm')"
      :description="t('settings.storage.backup.restoreConfirmDesc')"
      :confirm-text="t('settings.storage.backup.restore')"
      confirm-color="red"
      @confirm="performRestore"
    />

  </Teleport>

  <Teleport to="body">
    <div v-if="showDataMigrateDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showDataMigrateDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('settings.storage.migrateTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-secondary mb-3">
          {{ t('settings.storage.migrateDescription') }}
        </p>
        <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 mb-3 text-xs font-mono space-y-1">
          <div class="text-red-500 dark:text-red-400">{{ t('settings.storage.migrateFrom') }}: {{ storagePaths?.app_data_dir }}</div>
          <div class="text-green-500 dark:text-green-400">{{ t('settings.storage.migrateTo') }}: {{ pendingDataDir }}</div>
        </div>
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
          <p class="text-xs text-yellow-700 dark:text-yellow-400">{{ t('settings.storage.migrateWarning') }}</p>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showDataMigrateDialog = false; saveSettings()" :disabled="isMigratingData" class="btn-secondary">{{ t('settings.pluginRepo.skipMigration') }}</button>
          <button @click="executeDataMigration" :disabled="isMigratingData" class="btn-primary disabled:opacity-50">
            {{ isMigratingData ? t('settings.storage.migrating') : t('settings.storage.migrateButton') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showResetConfirm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click="showResetConfirm = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md max-h-[90vh] overflow-y-auto shadow-xl" @click.stop>
        <div class="flex items-center gap-3 mb-6">
          <div class="w-10 h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
            <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('settings.resetDataConfirmTitle') }}</h3>
        </div>

        <div class="flex items-center justify-center gap-2 mb-6">
          <div :class="['w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold', resetStep >= 1 ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-surface-hover text-gray-500 dark:text-content-muted']">1</div>
          <div :class="['flex-1 h-1', resetStep >= 2 ? 'bg-primary-600' : 'bg-gray-200 dark:bg-surface-hover']"></div>
          <div :class="['w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold', resetStep >= 2 ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-surface-hover text-gray-500 dark:text-content-muted']">2</div>
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
          <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3 mb-4">
            <p class="text-sm text-blue-800 dark:text-blue-300">
              {{ t('settings.resetDataAutoBackup') }}
            </p>
          </div>
          <button @click="goToStep(2)" class="w-full btn-primary">
            {{ t('settings.resetDataStep1Continue') }}
          </button>
        </div>

        <div v-if="resetStep === 2" class="mb-6">
          <p class="text-sm text-gray-600 dark:text-content-secondary mb-4">
            {{ t('settings.resetDataStep2NewDesc') }}
          </p>
          <div class="flex gap-3">
            <input
              v-model="backupFingerprint"
              type="text"
              :placeholder="t('settings.resetDataStep2NewPlaceholder')"
              class="flex-1 px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary placeholder-gray-400"
            />
            <button @click="selectResetBackupPath" class="px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-gray-50 dark:bg-surface-hover hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary transition-colors">
              {{ t('settings.buttons.select') }}
            </button>
          </div>
          <div class="flex justify-end gap-3 mt-4">
            <button @click="goToStep(1)" class="btn-secondary">{{ t('common.back') }}</button>
            <button @click="performReset" :disabled="isResetting || !backupFingerprint.trim()" class="btn-primary disabled:opacity-50">
              {{ isResetting ? t('settings.resetting') : t('settings.confirmReset') }}
            </button>
          </div>
        </div>

        <button @click="showResetConfirm = false" class="w-full mt-4 text-sm text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-gray-200">
          {{ t('common.cancel') }}
        </button>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showMirrorDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showMirrorDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ editingMirror ? t('settings.engineMirror.editMirror') : t('settings.engineMirror.addMirror') }}</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.engineMirror.mirrorName') }}</label>
            <input
              v-model="mirrorFormName"
              type="text"
              :placeholder="t('settings.engineMirror.mirrorNamePlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.engineMirror.mirrorUrl') }}</label>
            <input
              v-model="mirrorFormUrl"
              type="text"
              :placeholder="t('settings.engineMirror.mirrorUrlPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            />
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.engineMirror.urlHint') }}</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('settings.engineMirror.mirrorType') }}</label>
            <select
              v-model="mirrorFormType"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            >
              <option value="github_api">GitHub API</option>
              <option value="direct">{{ t('settings.engineMirror.mirrorTypeDirect') }}</option>
            </select>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('settings.engineMirror.mirrorTypeHint') }}</p>
          </div>
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="mirrorFormEnabled" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('settings.engineMirror.enableMirror') }}</span>
          </label>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showMirrorDialog = false"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveMirror"
            :disabled="!mirrorFormName.trim() || !mirrorFormUrl.trim()"
            class="btn-primary disabled:opacity-50"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showUnsavedDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showUnsavedDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('settings.unsavedTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-6">{{ t('settings.unsavedDesc') }}</p>
        <div class="flex justify-end gap-3">
          <button @click="discardChanges" class="px-4 py-2 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm">{{ t('settings.discardChanges') }}</button>
          <button @click="saveAndLeave" class="btn-primary text-sm">{{ t('settings.saveAndLeave') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>