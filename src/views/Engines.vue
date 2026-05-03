<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { Engine, RemoteEngineVersion, EngineMirrorConfig, EngineDownloadProgress, EngineReleaseChannel } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useAutoSetup } from '@/composables/useAutoSetup'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const { isRunning: isAutoSetupRunning, stepMessage: autoSetupMessage } = useAutoSetup()
const { t } = useI18n()
const engines = ref<Engine[]>([])
const isLoading = ref(false)
const showAddDialog = ref(false)
const newEnginePath = ref('')
const newEngineName = ref('')
const isRegistering = ref(false)
const showDeleteConfirm = ref(false)
const deleteAlsoFiles = ref(false)
const deleteTargetId = ref('')
const deleteBoundProjects = ref<string[]>([])
let unlistenDiscover: UnlistenFn | null = null
let unlistenDownloadProgress: UnlistenFn | null = null

const searchQuery = ref('')
const filterType = ref<string>('all')
const engineHealthMap = ref<Map<string, boolean>>(new Map())
const boundProjectsMap = ref<Map<string, string[]>>(new Map())
const expandedEngineId = ref<string | null>(null)

const showRenameDialog = ref(false)
const renameEngineId = ref('')
const renameInput = ref('')

const showDownloadDialog = ref(false)
const isFetchingVersions = ref(false)
const remoteVersions = ref<RemoteEngineVersion[]>([])
const selectedMirrorId = ref('official')
const mirrorConfigs = ref<EngineMirrorConfig[]>([])
const downloadChannelFilter = ref<EngineReleaseChannel | 'all'>('all')
const downloadVariantFilter = ref<'all' | 'standard' | 'mono'>('all')
const downloadSearchQuery = ref('')
const hideInstalled = ref(false)
const activeDownloads = ref<Map<string, EngineDownloadProgress>>(new Map())
const expandedReleaseVersion = ref<string>('')
const openMenuId = ref<string>('')
const collapsedGroups = ref<Set<string>>(new Set())

useDialogEscape(showAddDialog)
useDialogEscape(showRenameDialog)
useDialogEscape(showDownloadDialog)

onMounted(async () => {
  await loadEngines()
  try {
    const activeList = await api.getActiveDownloads()
    const newMap = new Map<string, EngineDownloadProgress>()
    for (const progress of activeList) {
      const key = `${progress.version}_${progress.variant}`
      newMap.set(key, progress)
    }
    activeDownloads.value = newMap
  } catch {}
  unlistenDiscover = await listen('engines-discovered', () => {
    loadEngines()
  })
  unlistenDownloadProgress = await listen('engine-download-progress', (event) => {
    const progress = event.payload as EngineDownloadProgress
    const key = `${progress.version}_${progress.variant}`
    const newMap = new Map(activeDownloads.value)
    if (progress.stage === 'complete') {
      newMap.delete(key)
    } else {
      newMap.set(key, progress)
    }
    activeDownloads.value = newMap
  })
  document.addEventListener('click', handleGlobalClick)
})

onUnmounted(() => {
  if (unlistenDiscover) {
    unlistenDiscover()
  }
  if (unlistenDownloadProgress) {
    unlistenDownloadProgress()
  }
  document.removeEventListener('click', handleGlobalClick)
})

const defaultEngine = computed(() => {
  return engines.value.find(e => e.is_default)
})

const filteredEngines = computed(() => {
  return engines.value.filter(engine => {
    const matchesSearch = searchQuery.value === '' ||
      engine.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      engine.version.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      engine.path.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesType = filterType.value === 'all' ||
      engine.engine_type === filterType.value

    return matchesSearch && matchesType
  })
})

const filteredRemoteVersions = computed(() => {
  return remoteVersions.value.filter(v => {
    const matchesChannel = downloadChannelFilter.value === 'all' || v.channel === downloadChannelFilter.value
    const matchesVariant = downloadVariantFilter.value === 'all' || v.variant === downloadVariantFilter.value
    const matchesSearch = downloadSearchQuery.value === '' ||
      v.version.toLowerCase().includes(downloadSearchQuery.value.toLowerCase()) ||
      v.tag_name.toLowerCase().includes(downloadSearchQuery.value.toLowerCase()) ||
      v.channel.toLowerCase().includes(downloadSearchQuery.value.toLowerCase())
    const matchesInstalled = !hideInstalled.value || !v.is_installed
    return matchesChannel && matchesVariant && matchesSearch && matchesInstalled
  })
})

const groupedRemoteVersions = computed(() => {
  const groups = new Map<string, RemoteEngineVersion[]>()
  for (const v of filteredRemoteVersions.value) {
    const key = `${v.major}.${v.minor}`
    if (!groups.has(key)) {
      groups.set(key, [])
    }
    groups.get(key)!.push(v)
  }
  return groups
})

const subGroupedVersions = (versions: RemoteEngineVersion[]) => {
  const subGroups = new Map<string, RemoteEngineVersion[]>()
  for (const v of versions) {
    const baseVersion = v.version.split('-')[0]
    const key = `${baseVersion}-${v.channel}`
    if (!subGroups.has(key)) {
      subGroups.set(key, [])
    }
    subGroups.get(key)!.push(v)
  }
  return subGroups
}

const channelBadgeClass = (channel: EngineReleaseChannel) => {
  switch (channel) {
    case 'Stable': return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
    case 'Rc': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400'
    case 'Beta': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400'
    case 'Alpha': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'
    case 'Dev': return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
    default: return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
  }
}

const channelLabel = (channel: EngineReleaseChannel, channelNumber?: number) => {
  const base = (() => {
    switch (channel) {
      case 'Stable': return t('engines.download.channelStable')
      case 'Rc': return t('engines.download.channelRc')
      case 'Beta': return t('engines.download.channelBeta')
      case 'Alpha': return t('engines.download.channelAlpha')
      case 'Dev': return t('engines.download.channelDev')
      default: return channel
    }
  })()
  if (channelNumber && channelNumber > 0 && channel !== 'Stable') {
    return `${base} ${channelNumber}`
  }
  return base
}

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '—'
  const mb = bytes / 1024 / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

const formatDate = (dateStr: string) => {
  try {
    return new Date(dateStr).toLocaleDateString()
  } catch {
    return dateStr
  }
}

const loadEngines = async () => {
  isLoading.value = true
  try {
    const result = await api.getEngines()
    engines.value = result
    await Promise.allSettled([checkAllEngineHealth(), loadAllBoundProjects()])
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const checkAllEngineHealth = async () => {
  const healthMap = new Map<string, boolean>()
  const results = await Promise.allSettled(
    engines.value.map(async (engine) => {
      try {
        const healthy = await api.checkEngineHealth(engine.engine_id)
        return { id: engine.engine_id, healthy }
      } catch {
        return { id: engine.engine_id, healthy: false }
      }
    })
  )
  for (const result of results) {
    if (result.status === 'fulfilled') {
      healthMap.set(result.value.id, result.value.healthy)
    }
  }
  engineHealthMap.value = healthMap
}

const loadAllBoundProjects = async () => {
  const projectsMap = new Map<string, string[]>()
  const results = await Promise.allSettled(
    engines.value.map(async (engine) => {
      try {
        const projects = await api.getEngineBoundProjects(engine.engine_id)
        return { id: engine.engine_id, projects }
      } catch {
        return { id: engine.engine_id, projects: [] }
      }
    })
  )
  for (const result of results) {
    if (result.status === 'fulfilled') {
      projectsMap.set(result.value.id, result.value.projects)
    }
  }
  boundProjectsMap.value = projectsMap
}

const discoverEngines = async () => {
  isLoading.value = true
  try {
    const discovered = await api.autoDiscoverEngines()
    if (discovered.length > 0) {
      toast.success(t('engines.discoveredCount', { count: discovered.length }))
      await loadEngines()
    } else {
      toast.info(t('engines.noNewEngines'))
    }
  } catch (error) {
    toast.error(t('engines.discoverFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const selectEnginePath = async () => {
  try {
    let selected = await open({
      directory: true,
      multiple: false,
      title: t('engines.selectEngineDir')
    })
    if (!selected) {
      selected = await open({
        multiple: false,
        title: t('engines.selectEngineFile'),
        filters: [{
          name: t('engines.engineExecutable'),
          extensions: ['exe', '*']
        }]
      })
    }
    if (selected && typeof selected === 'string') {
      newEnginePath.value = selected
    }
  } catch (error) {
    toast.error(t('common.selectDirFailed', { error }))
  }
}

const registerEngine = async () => {
  if (!newEnginePath.value) {
    toast.warning(t('engines.selectEngineDirFirst'))
    return
  }
  isRegistering.value = true
  try {
    const result = await api.registerEngine(newEnginePath.value, newEngineName.value)
    toast.success(t('engines.registerSuccess', { name: result.name }))
    showAddDialog.value = false
    newEnginePath.value = ''
    newEngineName.value = ''
    await loadEngines()
  } catch (error) {
    toast.error(t('engines.registerFailed', { error }))
  } finally {
    isRegistering.value = false
  }
}

const confirmRemoveEngine = async (engineId: string) => {
  deleteTargetId.value = engineId
  deleteAlsoFiles.value = false
  try {
    deleteBoundProjects.value = await api.getEngineBoundProjects(engineId)
  } catch {
    deleteBoundProjects.value = []
  }
  showDeleteConfirm.value = true
}

const onRemoveEngineConfirm = async () => {
  try {
    const removedEngine = engines.value.find(e => e.engine_id === deleteTargetId.value)
    await api.removeEngine(deleteTargetId.value, deleteAlsoFiles.value)
    toast.success(t('engines.deleteSuccess'))
    await loadEngines()
    if (removedEngine) {
      const localVersion = removedEngine.version.trim().toLowerCase()
      remoteVersions.value = remoteVersions.value.map(v => {
        const remoteBase = v.version.split('-')[0].trim().toLowerCase()
        if (localVersion === remoteBase || localVersion === v.version.trim().toLowerCase()) {
          return { ...v, is_installed: false }
        }
        return v
      })
    }
  } catch (error) {
    toast.error(t('common.deleteFailed', { error }))
  }
}

const setDefault = async (engineId: string) => {
  try {
    await api.setDefaultEngine(engineId)
    toast.success(t('engines.defaultSet'))
    await loadEngines()
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const openInFileManager = async (path: string) => {
  try {
    await api.openInFileManager(path)
  } catch (error) {
    toast.error(t('engines.openInFileManagerFailed', { error }))
  }
}

const openRenameDialog = (engine: Engine) => {
  renameEngineId.value = engine.engine_id
  renameInput.value = engine.name
  showRenameDialog.value = true
}

const saveRename = async () => {
  if (!renameInput.value.trim()) {
    toast.warning(t('engines.nameRequired'))
    return
  }
  try {
    await api.renameEngine(renameEngineId.value, renameInput.value)
    toast.success(t('engines.renameSuccess'))
    showRenameDialog.value = false
    await loadEngines()
  } catch (error) {
    toast.error(t('engines.renameFailed', { error }))
  }
}

const checkEngineUpdates = async () => {
  try {
    const result = await api.checkGodotUpdates()
    if (result.updates_available.length > 0) {
      toast.info(t('engines.updatesAvailable', { count: result.updates_available.length }))
    } else {
      toast.success(t('engines.noUpdates'))
    }
  } catch (error) {
    toast.error(t('engines.checkUpdatesFailed', { error }))
  }
}

const toggleBoundProjects = (engineId: string) => {
  expandedEngineId.value = expandedEngineId.value === engineId ? null : engineId
}

const openDownloadDialog = async () => {
  showDownloadDialog.value = true
  downloadChannelFilter.value = 'all'
  downloadVariantFilter.value = 'all'
  downloadSearchQuery.value = ''
  hideInstalled.value = false
  expandedReleaseVersion.value = ''

  try {
    await api.cleanupDownloadTemp()
  } catch { /* ignore */ }

  try {
    const settings = await api.getSettings()
    mirrorConfigs.value = settings.engine_mirrors || []
    if (settings.selected_mirror_id && mirrorConfigs.value.find(m => m.id === settings.selected_mirror_id && m.enabled)) {
      selectedMirrorId.value = settings.selected_mirror_id
    } else if (mirrorConfigs.value.length > 0) {
      const firstEnabled = mirrorConfigs.value.find(m => m.enabled)
      selectedMirrorId.value = firstEnabled?.id || mirrorConfigs.value[0].id
    }
  } catch {
    mirrorConfigs.value = []
  }

  if (remoteVersions.value.length === 0) {
    await fetchRemoteVersions()
  }
}

const fetchRemoteVersions = async (forceRefresh: boolean = false) => {
  if (!selectedMirrorId.value) return
  isFetchingVersions.value = true
  if (forceRefresh) {
    remoteVersions.value = []
  }
  try {
    const settings = await api.getSettings()
    if (settings.selected_mirror_id !== selectedMirrorId.value) {
      settings.selected_mirror_id = selectedMirrorId.value
      await api.saveSettings(settings)
    }
  } catch { /* ignore */ }
  try {
    const versions = await api.fetchRemoteEngineVersions(selectedMirrorId.value, forceRefresh)
    remoteVersions.value = versions
    initCollapsedGroups()
  } catch (error) {
    const errMsg = String(error)
    if (errMsg.includes('RATE_LIMITED')) {
      toast.error(t('engines.download.rateLimitError'))
    } else if (errMsg.includes('NETWORK_ERROR')) {
      toast.error(t('engines.download.networkError'))
    } else {
      toast.error(t('engines.download.fetchVersionsFailed', { error }))
    }
  } finally {
    isFetchingVersions.value = false
  }
}

const startDownload = async (version: RemoteEngineVersion) => {
  const dlKey = `${version.version}_${version.variant}`
  if (activeDownloads.value.has(dlKey)) {
    toast.info(t('engines.download.alreadyDownloading'))
    return
  }
  const newMap = new Map(activeDownloads.value)
  newMap.set(dlKey, {
    version: version.version,
    variant: version.variant,
    stage: 'downloading',
    downloaded_bytes: 0,
    total_bytes: 0,
    progress: 0,
    message: t('engines.download.downloading')
  })
  activeDownloads.value = newMap
  try {
    const result = await api.downloadEngine(version)
    if (result.cancelled) {
      toast.info(t('engines.download.downloadCancelled'))
    } else if (result.success && result.engine) {
      toast.success(t('engines.download.downloadSuccess', { name: result.engine.name }))
      remoteVersions.value = remoteVersions.value.map(v => {
        if (v.version === version.version && v.variant === version.variant) {
          return { ...v, is_installed: true }
        }
        return v
      })
    } else if (result.error) {
      toast.error(t('engines.download.downloadFailed', { error: result.error }))
    }
  } catch (error) {
    toast.error(t('engines.download.downloadFailed', { error }))
  } finally {
    const cleanupMap = new Map(activeDownloads.value)
    cleanupMap.delete(dlKey)
    activeDownloads.value = cleanupMap
  }
}

const cancelDownload = async (version: string, variant: string) => {
  try {
    await api.cancelEngineDownload(version, variant)
    toast.info(t('engines.download.cancelled'))
  } catch (error) {
    toast.error(t('engines.download.cancelFailed', { error }))
  }
}

const handleDownloadDialogClose = () => {
  if (activeDownloads.value.size > 0) {
    toast.info(t('engines.download.downloadInBackground'))
  }
  showDownloadDialog.value = false
}

const onMirrorChange = () => {
  remoteVersions.value = []
  fetchRemoteVersions(false)
}

const handleLaunchEngine = async (engineId: string) => {
  try {
    await api.launchEngine(engineId)
    toast.success(t('engines.launched'))
  } catch (error) {
    toast.error(t('engines.launchFailed', { error }))
  }
}

const toggleEngineMenu = (engineId: string) => {
  openMenuId.value = openMenuId.value === engineId ? '' : engineId
}

const handleGlobalClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.engine-menu-wrapper')) {
    openMenuId.value = ''
  }
}

const toggleGroup = (groupKey: string) => {
  const newSet = new Set(collapsedGroups.value)
  if (newSet.has(groupKey)) {
    newSet.delete(groupKey)
  } else {
    newSet.add(groupKey)
  }
  collapsedGroups.value = newSet
}

const initCollapsedGroups = () => {
  const keys = new Set<string>()
  let first = true
  for (const [groupKey] of groupedRemoteVersions.value) {
    if (first) {
      first = false
      continue
    }
    keys.add(groupKey)
  }
  collapsedGroups.value = keys
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="shrink-0 space-y-4 pb-4">
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('engines.title') }}</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="discoverEngines"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('engines.discover') }}
        </button>
        <button
          @click="openDownloadDialog"
          class="px-4 py-2 border border-primary-600 text-primary-600 dark:text-primary-400 bg-white dark:bg-gray-800 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors text-sm inline-flex items-center gap-1.5"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          {{ t('engines.download.title') }}
        </button>
        <button
          @click="showAddDialog = true"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
        >
          {{ t('engines.register') }}
        </button>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <svg class="w-5 h-5 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('engines.defaultEngine') }}:</span>
          </div>
          <span v-if="defaultEngine" class="text-sm font-medium text-gray-900 dark:text-gray-100">
            {{ defaultEngine.name }} (v{{ defaultEngine.version }})
          </span>
          <span v-else class="text-sm text-yellow-600 dark:text-yellow-400">
            {{ t('engines.noDefaultEngine') }}
          </span>
        </div>
        <button
          @click="checkEngineUpdates"
          class="px-3 py-1.5 text-xs font-medium border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
        >
          {{ t('engines.checkUpdates') }}
        </button>
      </div>
    </div>

    <div v-if="activeDownloads.size > 0 && !showDownloadDialog" class="bg-blue-50 dark:bg-blue-900/20 rounded-xl shadow p-4">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm font-medium text-blue-800 dark:text-blue-300">
          {{ t('engines.download.downloading') }} ({{ activeDownloads.size }})
        </span>
        <button
          @click="openDownloadDialog"
          class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
        >
          {{ t('engines.download.title') }}
        </button>
      </div>
      <div class="space-y-2">
        <div v-for="[key, progress] in activeDownloads" :key="key">
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs text-blue-700 dark:text-blue-300">v{{ progress.version }}{{ progress.variant === 'mono' ? ' (.NET)' : '' }} - {{ progress.message }}</span>
            <div class="flex items-center gap-2">
              <span class="text-xs text-blue-600 dark:text-blue-400">{{ progress.progress.toFixed(1) }}%</span>
              <button
                @click="cancelDownload(progress.version, progress.variant)"
                class="text-xs text-red-600 dark:text-red-400 hover:underline"
              >
                {{ t('engines.download.cancel') }}
              </button>
            </div>
          </div>
          <div class="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-1.5">
            <div class="bg-blue-600 dark:bg-blue-400 h-1.5 rounded-full transition-all duration-300" :style="{ width: `${progress.progress}%` }"></div>
          </div>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="flex flex-col lg:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('engines.search')"
            class="w-full px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          />
        </div>
        <div class="flex gap-2 items-center">
          <select
            v-model="filterType"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">{{ t('engines.allTypes') }}</option>
            <option value="Godot4">Godot 4</option>
            <option value="Godot3">Godot 3</option>
            <option value="Unknown">{{ t('engines.unknown') }}</option>
          </select>
        </div>
      </div>
    </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="isAutoSetupRunning && engines.length === 0" class="text-center py-16">
      <div class="animate-spin rounded-full h-10 w-10 border-2 border-primary-600 border-t-transparent mx-auto"></div>
      <h3 class="mt-4 text-sm font-medium text-gray-900 dark:text-gray-100">{{ autoSetupMessage }}</h3>
      <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">{{ t('autoSetup.pleaseWait') }}</p>
    </div>

    <div v-else-if="engines.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('engines.empty') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        {{ t('engines.emptyDesc') }}
      </p>
      <div class="mt-4 flex justify-center gap-3">
        <button
          @click="discoverEngines"
          :disabled="isLoading"
          class="inline-flex items-center gap-1.5 px-4 py-2 border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          {{ t('engines.discover') }}
        </button>
        <button
          @click="openDownloadDialog"
          class="inline-flex items-center gap-1.5 px-4 py-2 border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          {{ t('engines.download.title') }}
        </button>
        <button
          @click="showAddDialog = true"
          class="inline-flex items-center gap-1.5 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t('engines.register') }}
        </button>
      </div>
    </div>

    <div v-else-if="filteredEngines.length === 0" class="text-center py-12">
      <p class="text-sm text-gray-500 dark:text-gray-400">{{ t('engines.noMatchingEngines') }}</p>
    </div>

    <div v-else class="flex-1 min-h-0 bg-white dark:bg-gray-800 rounded-xl shadow overflow-hidden">
      <div class="overflow-x-hidden h-full overflow-y-auto">
        <table class="w-full min-w-[800px]">
          <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
            <template v-for="engine in filteredEngines" :key="engine.engine_id">
            <tr
              :class="[
                'hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors',
                engine.is_default ? 'bg-primary-50/50 dark:bg-primary-900/10' : ''
              ]"
            >
              <td class="px-4 py-4 whitespace-nowrap">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-lg bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center">
                    <svg class="w-5 h-5 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                  </div>
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-gray-900 dark:text-gray-100 text-sm">
                        {{ engine.name }}
                      </span>
                      <span
                        v-if="engine.is_default"
                        class="px-2 py-0.5 rounded text-xs font-medium bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-400"
                      >
                        {{ t('engines.default') }}
                      </span>
                    </div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">v{{ engine.version }}</span>
                    <span
                      v-if="engine.version.toLowerCase().includes('mono')"
                      class="px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400"
                    >
                      .NET
                    </span>
                  </div>
                </div>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <span class="px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300">
                  {{ engine.engine_type === 'Godot4' ? 'Godot 4' : engine.engine_type === 'Godot3' ? 'Godot 3' : t('engines.unknown') }}
                </span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <span
                  v-if="engineHealthMap.get(engine.engine_id) === true"
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                >
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  {{ t('engines.healthy') }}
                </span>
                <span
                  v-else-if="engineHealthMap.get(engine.engine_id) === false"
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
                >
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                  {{ t('engines.unhealthy') }}
                </span>
                <span v-else class="text-xs text-gray-400">{{ t('engines.checking') }}</span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <button
                  @click="toggleBoundProjects(engine.engine_id)"
                  class="text-sm text-primary-600 dark:text-primary-400 hover:underline cursor-pointer"
                  :title="t('engines.boundProjectsList')"
                >
                  {{ boundProjectsMap.get(engine.engine_id)?.length || 0 }} {{ t('engines.projectCount') }}
                  <svg class="w-3 h-3 inline-block ml-0.5 transition-transform" :class="{ 'rotate-180': expandedEngineId === engine.engine_id }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
                </button>
              </td>
              <td class="px-4 py-4">
                <span
                  class="text-sm text-primary-600 dark:text-primary-400 hover:underline cursor-pointer truncate max-w-xs block"
                  :title="engine.path"
                  @click="openInFileManager(engine.path)"
                >
                  {{ engine.path }}
                </span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <div class="flex items-center justify-end gap-1">
                  <button
                    v-if="engineHealthMap.get(engine.engine_id) === true"
                    @click="handleLaunchEngine(engine.engine_id)"
                    class="text-green-600 hover:text-green-800 dark:text-green-400 p-2 rounded-lg hover:bg-green-50 dark:hover:bg-green-900/20 transition-colors"
                    :title="t('engines.launchEngine')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  </button>
                  <button
                    v-if="!engine.is_default"
                    @click="setDefault(engine.engine_id)"
                    class="text-primary-600 hover:text-primary-800 dark:text-primary-400 p-2 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors"
                    :title="t('engines.setDefault')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                    </svg>
                  </button>
                  <div class="engine-menu-wrapper" style="position: relative; display: inline-block">
                    <button
                      @click="toggleEngineMenu(engine.engine_id)"
                      class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                      :title="t('engines.moreActions')"
                    >
                      <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                      </svg>
                    </button>
                    <div
                      v-if="openMenuId === engine.engine_id"
                      class="absolute right-0 top-full mt-1 bg-white dark:bg-gray-700 rounded-lg shadow-lg border border-gray-200 dark:border-gray-600 py-1 z-20 min-w-[140px]"
                    >
                      <button
                        v-if="engineHealthMap.get(engine.engine_id) === true"
                        @click="handleLaunchEngine(engine.engine_id); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-green-700 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                        {{ t('engines.launchEngine') }}
                      </button>
                      <button
                        @click="openRenameDialog(engine); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-600 flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
                        {{ t('engines.rename') }}
                      </button>
                      <button
                        @click="openInFileManager(engine.path); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-600 flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
                        {{ t('engines.openInFileManager') }}
                      </button>
                      <hr class="my-1 border-gray-200 dark:border-gray-600" />
                      <button
                        @click="confirmRemoveEngine(engine.engine_id); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                        {{ t('engines.deleteEngine') }}
                      </button>
                    </div>
                  </div>
                </div>
              </td>
            </tr>
            <tr v-if="expandedEngineId === engine.engine_id">
              <td colspan="6" class="px-6 py-3 bg-gray-50 dark:bg-gray-700/30">
                <div class="text-sm">
                  <span class="font-medium text-gray-700 dark:text-gray-300">{{ t('engines.boundProjectsList') }}:</span>
                  <div v-if="boundProjectsMap.get(engine.engine_id)?.length" class="mt-1 flex flex-wrap gap-2">
                    <span
                      v-for="projectName in boundProjectsMap.get(engine.engine_id)"
                      :key="projectName"
                      class="px-2 py-1 bg-white dark:bg-gray-600 rounded text-xs text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-500"
                    >
                      {{ projectName }}
                    </span>
                  </div>
                  <span v-else class="text-gray-400 dark:text-gray-500 ml-2">{{ t('engines.noBoundProjects') }}</span>
                </div>
              </td>
            </tr>
            </template>
          </tbody>
        </table>
      </div>
    </div>
  </div>

  <Teleport to="body">
  <div v-if="showAddDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showAddDialog = false; newEnginePath = ''; newEngineName = ''">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('engines.registerTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          {{ t('engines.registerDesc') }}
        </p>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('engines.engineName') }}</label>
            <input
              v-model="newEngineName"
              type="text"
              :placeholder="t('engines.engineNamePlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('engines.enginePath') }}</label>
            <div class="flex gap-2">
              <input
                v-model="newEnginePath"
                type="text"
                readonly
                :placeholder="t('engines.enginePathPlaceholder')"
                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
              />
              <button
                @click="selectEnginePath"
                class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
              >
                {{ t('projects.browse') }}
              </button>
            </div>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showAddDialog = false; newEnginePath = ''; newEngineName = ''"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="registerEngine"
            :disabled="isRegistering || !newEnginePath"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ isRegistering ? t('engines.registering') : t('engines.register') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showDownloadDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="handleDownloadDialogClose">
      <div class="bg-white dark:bg-gray-800 rounded-lg w-full max-w-3xl shadow-xl max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center p-6 pb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('engines.download.title') }}</h3>
          <button @click="handleDownloadDialogClose" class="text-sm text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            {{ t('common.close') }}
          </button>
        </div>

        <div class="px-6 pb-4 space-y-3">
          <div class="flex gap-3 items-end">
            <div class="flex-1">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('engines.download.mirror') }}</label>
              <select
                v-model="selectedMirrorId"
                @change="onMirrorChange"
                :disabled="isFetchingVersions"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
              >
                <option v-for="mirror in mirrorConfigs" :key="mirror.id" :value="mirror.id" :disabled="!mirror.enabled">
                  {{ mirror.name }}{{ !mirror.enabled ? ` (${t('engines.download.disabled')})` : '' }}
                </option>
              </select>
            </div>
            <button
              @click="fetchRemoteVersions(true)"
              :disabled="isFetchingVersions"
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 text-sm whitespace-nowrap disabled:opacity-50"
            >
              {{ isFetchingVersions ? t('engines.download.fetching') : t('engines.download.refresh') }}
            </button>
          </div>

          <div class="flex gap-3">
            <div class="flex-1">
              <input
                v-model="downloadSearchQuery"
                type="text"
                :placeholder="t('engines.download.searchPlaceholder')"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
              />
            </div>
            <select
              v-model="downloadChannelFilter"
              class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            >
              <option value="all">{{ t('engines.download.allChannels') }}</option>
              <option value="Stable">{{ t('engines.download.channelStable') }}</option>
              <option value="Rc">{{ t('engines.download.channelRc') }}</option>
              <option value="Beta">{{ t('engines.download.channelBeta') }}</option>
              <option value="Alpha">{{ t('engines.download.channelAlpha') }}</option>
              <option value="Dev">{{ t('engines.download.channelDev') }}</option>
            </select>
            <select
              v-model="downloadVariantFilter"
              class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            >
              <option value="all">{{ t('engines.download.allVariants') }}</option>
              <option value="standard">{{ t('engines.download.variantStandard') }}</option>
              <option value="mono">{{ t('engines.download.variantMono') }}</option>
            </select>
            <label class="flex items-center gap-1.5 text-sm text-gray-600 dark:text-gray-400 cursor-pointer whitespace-nowrap">
              <input type="checkbox" v-model="hideInstalled" class="rounded border-gray-300 text-primary-600 focus:ring-primary-500" />
              {{ t('engines.download.hideInstalled') }}
            </label>
          </div>

          <div v-if="activeDownloads.size > 0" class="space-y-2">
            <div v-for="[key, progress] in activeDownloads" :key="key" class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3">
              <div class="flex justify-between items-center mb-1">
                <span class="text-sm font-medium text-blue-800 dark:text-blue-300">v{{ progress.version }}{{ progress.variant === 'mono' ? ' (.NET)' : '' }} - {{ progress.message }}</span>
                <span class="text-xs text-blue-600 dark:text-blue-400">{{ progress.progress.toFixed(1) }}%</span>
              </div>
              <div class="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-2">
                <div class="bg-blue-600 dark:bg-blue-400 h-2 rounded-full transition-all duration-300" :style="{ width: `${progress.progress}%` }"></div>
              </div>
              <div v-if="progress.total_bytes > 0" class="text-xs text-blue-500 dark:text-blue-400 mt-1">
                {{ formatFileSize(progress.downloaded_bytes) }} / {{ formatFileSize(progress.total_bytes) }}
              </div>
              <div class="flex justify-end mt-2">
                <button
                  @click="cancelDownload(progress.version, progress.variant)"
                  class="px-3 py-1 text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300"
                >
                  {{ t('engines.download.cancel') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto px-6 pb-6">
          <div v-if="isFetchingVersions" class="flex justify-center py-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
          </div>

          <div v-else-if="remoteVersions.length === 0" class="text-center py-8">
            <p class="text-sm text-gray-500 dark:text-gray-400">{{ t('engines.download.noVersions') }}</p>
          </div>

          <div v-else-if="filteredRemoteVersions.length === 0" class="text-center py-8">
            <p class="text-sm text-gray-500 dark:text-gray-400">{{ t('engines.download.noMatchingVersions') }}</p>
          </div>

          <div v-else class="space-y-4">
            <div v-for="[groupKey, versions] in groupedRemoteVersions" :key="groupKey">
              <div
                class="sticky top-0 bg-white dark:bg-gray-800 py-1.5 px-3 -mx-3 border-b border-gray-200 dark:border-gray-700 mb-2 z-10 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                @click="toggleGroup(groupKey)"
              >
                <div class="flex items-center gap-2">
                  <svg class="w-3 h-3 text-gray-400 transition-transform" :class="{ '-rotate-90': collapsedGroups.has(groupKey) }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
                  <span class="text-sm font-semibold text-gray-700 dark:text-gray-300">Godot {{ groupKey }}</span>
                  <span class="text-xs text-gray-400 ml-2">{{ versions.length }} {{ t('engines.download.versionCount') }}</span>
                </div>
              </div>
              <div v-if="!collapsedGroups.has(groupKey)" class="space-y-2">
            <div v-for="[subKey, subVersions] in subGroupedVersions(versions)" :key="subKey">
              <div
                :class="[
                  'rounded-lg border transition-colors',
                  subVersions.every(v => v.is_installed)
                    ? 'bg-gray-50 dark:bg-gray-700/30 border-gray-200 dark:border-gray-600'
                    : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700'
                ]"
              >
                <div
                  v-for="(version, vIdx) in subVersions"
                  :key="`${version.tag_name}_${version.variant}`"
                  :class="[
                    vIdx > 0 ? 'border-t border-gray-100 dark:border-gray-700' : ''
                  ]"
                >
                  <div class="flex items-center gap-3 p-3">
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="font-medium text-sm text-gray-900 dark:text-gray-100">v{{ version.version }}</span>
                        <span :class="['px-1.5 py-0.5 rounded text-xs font-medium', channelBadgeClass(version.channel)]">
                          {{ channelLabel(version.channel, version.channel_number) }}
                        </span>
                        <span
                          v-if="version.is_lts"
                          class="px-1.5 py-0.5 rounded text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400"
                        >
                          LTS
                        </span>
                        <span
                          v-if="version.variant === 'mono'"
                          class="px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400"
                        >
                          .NET
                        </span>
                        <span
                          v-if="version.is_installed"
                          class="px-1.5 py-0.5 rounded text-xs font-medium bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-400"
                        >
                          {{ t('engines.download.installed') }}
                        </span>
                      </div>
                      <div class="flex items-center gap-3 mt-1 text-xs text-gray-500 dark:text-gray-400">
                        <span>{{ formatFileSize(version.file_size) }}</span>
                        <span>{{ formatDate(version.published_at) }}</span>
                        <a
                          v-if="version.release_url"
                          :href="version.release_url"
                          target="_blank"
                          rel="noopener noreferrer"
                          class="text-primary-600 dark:text-primary-400 hover:underline inline-flex items-center gap-0.5"
                        >
                          {{ t('engines.download.sourceLink') }}
                          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
                        </a>
                        <button
                          v-if="version.release_notes"
                          @click="expandedReleaseVersion = expandedReleaseVersion === `${version.version}_${version.variant}` ? '' : `${version.version}_${version.variant}`"
                          class="text-primary-600 dark:text-primary-400 hover:underline"
                        >
                          {{ expandedReleaseVersion === `${version.version}_${version.variant}` ? t('engines.download.hideNotes') : t('engines.download.showNotes') }}
                        </button>
                      </div>
                    </div>
                    <button
                      @click="startDownload(version)"
                      :disabled="activeDownloads.has(`${version.version}_${version.variant}`)"
                      :class="[
                        'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors whitespace-nowrap',
                        version.is_installed
                          ? 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
                          : activeDownloads.has(`${version.version}_${version.variant}`)
                            ? 'bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400'
                            : 'bg-primary-600 text-white hover:bg-primary-700 disabled:opacity-50'
                      ]"
                    >
                      <template v-if="activeDownloads.has(`${version.version}_${version.variant}`)">
                        {{ t('engines.download.downloading') }}
                      </template>
                      <template v-else-if="version.is_installed">
                        {{ t('engines.download.reDownload') }}
                      </template>
                      <template v-else>
                        {{ t('engines.download.downloadAction') }}
                      </template>
                    </button>
                  </div>
                  <div
                    v-if="expandedReleaseVersion === `${version.version}_${version.variant}` && version.release_notes"
                    class="px-3 pb-3"
                  >
                    <div class="p-2 bg-gray-50 dark:bg-gray-700/50 rounded text-xs text-gray-600 dark:text-gray-300 whitespace-pre-wrap max-h-40 overflow-y-auto">{{ version.release_notes }}</div>
                  </div>
                </div>
              </div>
            </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showRenameDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRenameDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('engines.renameTitle') }}</h3>
        <input
          v-model="renameInput"
          type="text"
          :placeholder="t('engines.engineNamePlaceholder')"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          @keyup.enter="saveRename"
        />
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showRenameDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveRename"
            :disabled="!renameInput.trim()"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('engines.deleteConfirm')"
      :description="deleteBoundProjects.length > 0 
        ? t('engines.deleteConfirmDescWithProjects', { projects: deleteBoundProjects.join(', ') }) 
        : t('engines.deleteConfirmDesc')"
      :confirm-text="t('common.confirm')"
      @confirm="onRemoveEngineConfirm"
    >
      <label class="flex items-center gap-2 mt-2 cursor-pointer">
        <input type="checkbox" v-model="deleteAlsoFiles" class="rounded border-gray-300 text-red-600 focus:ring-red-500" />
        <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('engines.deleteAlsoFiles') }}</span>
      </label>
      <p v-if="deleteAlsoFiles" class="mt-2 text-xs text-red-600 dark:text-red-400">{{ t('engines.deleteAlsoFilesWarning') }}</p>
    </ConfirmDialog>
  </Teleport>
</template>
