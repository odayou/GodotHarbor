<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { Engine, RemoteEngineVersion, EngineMirrorConfig, EngineDownloadProgress, EngineReleaseChannel } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const { t } = useI18n()
const engines = ref<Engine[]>([])
const isLoading = ref(false)
const showAddDialog = ref(false)
const newEnginePath = ref('')
const newEngineName = ref('')
const isRegistering = ref(false)
const showDeleteConfirm = ref(false)
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
const downloadSearchQuery = ref('')
const isDownloading = ref(false)
const downloadProgress = ref<EngineDownloadProgress | null>(null)
const downloadingVersion = ref<string>('')
const expandedReleaseVersion = ref<string>('')

useDialogEscape(showAddDialog)
useDialogEscape(showRenameDialog)
useDialogEscape(showDownloadDialog)

onMounted(async () => {
  await loadEngines()
  unlistenDiscover = await listen('engines-discovered', () => {
    loadEngines()
  })
  unlistenDownloadProgress = await listen('engine-download-progress', (event) => {
    downloadProgress.value = event.payload as EngineDownloadProgress
  })
})

onUnmounted(() => {
  if (unlistenDiscover) {
    unlistenDiscover()
  }
  if (unlistenDownloadProgress) {
    unlistenDownloadProgress()
  }
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
    const matchesSearch = downloadSearchQuery.value === '' ||
      v.version.toLowerCase().includes(downloadSearchQuery.value.toLowerCase()) ||
      v.tag_name.toLowerCase().includes(downloadSearchQuery.value.toLowerCase())
    return matchesChannel && matchesSearch
  })
})

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

const channelLabel = (channel: EngineReleaseChannel) => {
  switch (channel) {
    case 'Stable': return t('engines.download.channelStable')
    case 'Rc': return t('engines.download.channelRc')
    case 'Beta': return t('engines.download.channelBeta')
    case 'Alpha': return t('engines.download.channelAlpha')
    case 'Dev': return t('engines.download.channelDev')
    default: return channel
  }
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
    await checkAllEngineHealth()
    await loadAllBoundProjects()
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const checkAllEngineHealth = async () => {
  const healthMap = new Map<string, boolean>()
  for (const engine of engines.value) {
    try {
      const healthy = await api.checkEngineHealth(engine.engine_id)
      healthMap.set(engine.engine_id, healthy)
    } catch {
      healthMap.set(engine.engine_id, false)
    }
  }
  engineHealthMap.value = healthMap
}

const loadAllBoundProjects = async () => {
  const projectsMap = new Map<string, string[]>()
  for (const engine of engines.value) {
    try {
      const projects = await api.getEngineBoundProjects(engine.engine_id)
      projectsMap.set(engine.engine_id, projects)
    } catch {
      projectsMap.set(engine.engine_id, [])
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
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('engines.selectEngineDir')
    })
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
  try {
    deleteBoundProjects.value = await api.getEngineBoundProjects(engineId)
  } catch {
    deleteBoundProjects.value = []
  }
  showDeleteConfirm.value = true
}

const onRemoveEngineConfirm = async () => {
  try {
    await api.removeEngine(deleteTargetId.value)
    toast.success(t('engines.deleteSuccess'))
    await loadEngines()
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
  downloadSearchQuery.value = ''
  expandedReleaseVersion.value = ''

  if (!isDownloading.value) {
    remoteVersions.value = []
    downloadProgress.value = null
    downloadingVersion.value = ''
  }

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

  if (!isDownloading.value) {
    await fetchRemoteVersions()
  }
}

const fetchRemoteVersions = async () => {
  if (!selectedMirrorId.value) return
  isFetchingVersions.value = true
  remoteVersions.value = []
  try {
    const settings = await api.getSettings()
    if (settings.selected_mirror_id !== selectedMirrorId.value) {
      settings.selected_mirror_id = selectedMirrorId.value
      await api.saveSettings(settings)
    }
  } catch { /* ignore */ }
  try {
    const versions = await api.fetchRemoteEngineVersions(selectedMirrorId.value)
    remoteVersions.value = versions
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
  if (version.is_installed) {
    toast.info(t('engines.download.alreadyInstalled'))
    return
  }
  isDownloading.value = true
  downloadingVersion.value = version.version
  downloadProgress.value = null
  try {
    const result = await api.downloadEngine(version)
    if (result.cancelled) {
      toast.info(t('engines.download.downloadCancelled'))
    } else if (result.success && result.engine) {
      toast.success(t('engines.download.downloadSuccess', { name: result.engine.name }))
      await loadEngines()
      await fetchRemoteVersions()
    } else if (result.error) {
      toast.error(t('engines.download.downloadFailed', { error: result.error }))
    }
  } catch (error) {
    toast.error(t('engines.download.downloadFailed', { error }))
  } finally {
    isDownloading.value = false
    downloadingVersion.value = ''
    downloadProgress.value = null
  }
}

const cancelDownload = async () => {
  try {
    await api.cancelEngineDownload()
  } catch {
    // ignore
  }
}
</script>

<template>
  <div class="relative">
    <div class="space-y-6">
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

    <div v-if="isDownloading && !showDownloadDialog" class="bg-blue-50 dark:bg-blue-900/20 rounded-xl shadow p-4">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm font-medium text-blue-800 dark:text-blue-300">
          {{ t('engines.download.downloading') }} v{{ downloadingVersion }}
        </span>
        <div class="flex items-center gap-2">
          <span class="text-xs text-blue-600 dark:text-blue-400">
            {{ downloadProgress?.progress.toFixed(1) || 0 }}%
          </span>
          <button
            @click="openDownloadDialog"
            class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
          >
            {{ t('engines.download.title') }}
          </button>
          <button
            @click="cancelDownload"
            class="text-xs text-red-600 dark:text-red-400 hover:underline"
          >
            {{ t('engines.download.cancel') }}
          </button>
        </div>
      </div>
      <div class="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-2">
        <div class="bg-blue-600 dark:bg-blue-400 h-2 rounded-full transition-all duration-300" :style="{ width: `${downloadProgress?.progress || 0}%` }"></div>
      </div>
      <div v-if="downloadProgress?.message" class="text-xs text-blue-500 dark:text-blue-400 mt-1">
        {{ downloadProgress.message }}
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

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
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

    <div v-else class="bg-white dark:bg-gray-800 rounded-xl shadow overflow-hidden">
      <div class="overflow-x-auto">
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
                      <span
                        v-if="engineHealthMap.get(engine.engine_id) === false"
                        class="px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
                        :title="t('engines.exeNotFound')"
                      >
                        ⚠️
                      </span>
                    </div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">v{{ engine.version }}</span>
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
                  {{ boundProjectsMap.get(engine.engine_id)?.length || 0 }}
                  <svg class="w-3 h-3 inline-block ml-0.5 transition-transform" :class="{ 'rotate-180': expandedEngineId === engine.engine_id }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
                </button>
              </td>
              <td class="px-4 py-4">
                <span class="text-sm text-gray-500 dark:text-gray-400 truncate max-w-xs block" :title="engine.path">
                  {{ engine.path }}
                </span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <div class="flex items-center justify-end gap-1">
                  <button
                    @click="openRenameDialog(engine)"
                    class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                    :title="t('engines.rename')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  <button
                    @click="openInFileManager(engine.path)"
                    class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                    :title="t('engines.openInFileManager')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
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
                  <button
                    @click="confirmRemoveEngine(engine.engine_id)"
                    class="text-red-500 hover:text-red-700 p-2 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                    :title="t('engines.deleteEngine')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
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
    <div v-if="showDownloadDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showDownloadDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg w-full max-w-3xl shadow-xl max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center p-6 pb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('engines.download.title') }}</h3>
          <button @click="showDownloadDialog = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
          </button>
        </div>

        <div class="px-6 pb-4 space-y-3">
          <div class="flex gap-3 items-end">
            <div class="flex-1">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('engines.download.mirror') }}</label>
              <select
                v-model="selectedMirrorId"
                @change="fetchRemoteVersions"
                :disabled="isFetchingVersions"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
              >
                <option v-for="mirror in mirrorConfigs" :key="mirror.id" :value="mirror.id" :disabled="!mirror.enabled">
                  {{ mirror.name }}{{ !mirror.enabled ? ` (${t('engines.download.disabled')})` : '' }}
                </option>
              </select>
            </div>
            <button
              @click="fetchRemoteVersions"
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
          </div>

          <div v-if="isDownloading && downloadProgress" class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3">
            <div class="flex justify-between items-center mb-1">
              <span class="text-sm font-medium text-blue-800 dark:text-blue-300">{{ downloadProgress.message }}</span>
              <span class="text-xs text-blue-600 dark:text-blue-400">{{ downloadProgress.progress.toFixed(1) }}%</span>
            </div>
            <div class="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-2">
              <div class="bg-blue-600 dark:bg-blue-400 h-2 rounded-full transition-all duration-300" :style="{ width: `${downloadProgress.progress}%` }"></div>
            </div>
            <div v-if="downloadProgress.total_bytes > 0" class="text-xs text-blue-500 dark:text-blue-400 mt-1">
              {{ formatFileSize(downloadProgress.downloaded_bytes) }} / {{ formatFileSize(downloadProgress.total_bytes) }}
            </div>
            <div class="flex justify-end mt-2">
              <button
                @click="cancelDownload"
                class="px-3 py-1 text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300"
              >
                {{ t('engines.download.cancel') }}
              </button>
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

          <div v-else class="space-y-2">
            <div
              v-for="version in filteredRemoteVersions"
              :key="version.tag_name"
              :class="[
                'p-3 rounded-lg border transition-colors',
                version.is_installed
                  ? 'bg-gray-50 dark:bg-gray-700/30 border-gray-200 dark:border-gray-600'
                  : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-600'
              ]"
            >
              <div class="flex items-center gap-3">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-sm text-gray-900 dark:text-gray-100">v{{ version.version }}</span>
                    <span :class="['px-1.5 py-0.5 rounded text-xs font-medium', channelBadgeClass(version.channel)]">
                      {{ channelLabel(version.channel) }}
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
                    <span class="truncate" :title="version.file_name">{{ version.file_name }}</span>
                    <button
                      v-if="version.release_notes"
                      @click="expandedReleaseVersion = expandedReleaseVersion === version.version ? '' : version.version"
                      class="text-primary-600 dark:text-primary-400 hover:underline"
                    >
                      {{ expandedReleaseVersion === version.version ? t('engines.download.hideNotes') : t('engines.download.showNotes') }}
                    </button>
                  </div>
                </div>
                <button
                  @click="startDownload(version)"
                  :disabled="version.is_installed || isDownloading"
                  :class="[
                    'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors whitespace-nowrap',
                    version.is_installed
                      ? 'bg-gray-100 text-gray-400 dark:bg-gray-700 dark:text-gray-500 cursor-not-allowed'
                      : isDownloading && downloadingVersion === version.version
                        ? 'bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400'
                        : 'bg-primary-600 text-white hover:bg-primary-700 disabled:opacity-50'
                  ]"
                >
                  <template v-if="isDownloading && downloadingVersion === version.version">
                    {{ t('engines.download.downloading') }}
                  </template>
                  <template v-else-if="version.is_installed">
                    {{ t('engines.download.installed') }}
                  </template>
                  <template v-else>
                    {{ t('engines.download.downloadAction') }}
                  </template>
                </button>
              </div>
              <div
                v-if="expandedReleaseVersion === version.version && version.release_notes"
                class="mt-2 p-2 bg-gray-50 dark:bg-gray-700/50 rounded text-xs text-gray-600 dark:text-gray-300 whitespace-pre-wrap max-h-40 overflow-y-auto"
              >{{ version.release_notes }}</div>
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
    />
  </Teleport>
</template>
