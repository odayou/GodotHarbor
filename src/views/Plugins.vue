<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { Plugin, PluginUpdateInfo, PluginDependency, AssetLibrarySearchResult, AssetLibrarySearchResponse, AssetLibraryCategory, AssetLibraryAsset, PluginStorageStats, ProjectBinding, TotalStorageStats } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { usePluginStore } from '@/stores'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const pluginStore = usePluginStore()

const toast = useToast()
const { t } = useI18n()

const sendImportNotification = async (title: string, body: string) => {
  try {
    let permissionGranted = await isPermissionGranted()
    if (!permissionGranted) {
      const permission = await requestPermission()
      permissionGranted = permission === 'granted'
    }
    if (permissionGranted) {
      sendNotification({ title, body })
    }
  } catch (e) {
    console.error('Notification error:', e)
  }
}

const plugins = computed(() => pluginStore.plugins)
const isLoading = ref(false)
const hasLoaded = ref(false)
const gitUrl = ref('')
const showGitDialog = ref(false)
const showPluginDetail = ref(false)
const selectedPlugin = ref<Plugin | null>(null)
const pluginDependencies = ref<PluginDependency[]>([])
const pluginStorageStats = ref<PluginStorageStats | null>(null)
const pluginBindings = ref<ProjectBinding[]>([])
const showUpdatesDialog = ref(false)
const showImportModeDialog = ref(false)
const importMode = ref<'copy' | 'move' | 'reference'>('copy')
const totalStorageStats = ref<TotalStorageStats | null>(null)
const pluginUpdates = ref<PluginUpdateInfo[]>([])
const isCheckingUpdates = ref(false)

const searchQuery = ref('')
const filterCompatibility = ref<string>('all')
const filterSource = ref<string>('all')
const showFavoritesOnly = ref(false)

const selectedPluginIds = ref<Set<string>>(new Set())
const lastClickedPluginIndex = ref<number>(-1)
const isBatchMode = ref(false)

const togglePluginSelection = (plugin: Plugin, event: MouseEvent | Event) => {
  const mouseEvent = event as MouseEvent
  const pluginId = plugin.plugin_id
  const currentList = filteredPlugins.value
  const currentIndex = currentList.findIndex(p => p.plugin_id === pluginId)

  if (mouseEvent.shiftKey && lastClickedPluginIndex.value >= 0) {
    const start = Math.min(lastClickedPluginIndex.value, currentIndex)
    const end = Math.max(lastClickedPluginIndex.value, currentIndex)
    for (let i = start; i <= end; i++) {
      selectedPluginIds.value.add(currentList[i].plugin_id)
    }
  } else if (mouseEvent.ctrlKey || mouseEvent.metaKey) {
    if (selectedPluginIds.value.has(pluginId)) {
      selectedPluginIds.value.delete(pluginId)
    } else {
      selectedPluginIds.value.add(pluginId)
    }
  } else {
    if (selectedPluginIds.value.has(pluginId)) {
      selectedPluginIds.value.delete(pluginId)
      if (selectedPluginIds.value.size === 0) {
        isBatchMode.value = false
      }
    } else {
      selectedPluginIds.value.add(pluginId)
      isBatchMode.value = true
    }
  }

  lastClickedPluginIndex.value = currentIndex
  selectedPluginIds.value = new Set(selectedPluginIds.value)
}

const selectAllPlugins = () => {
  for (const p of filteredPlugins.value) {
    selectedPluginIds.value.add(p.plugin_id)
  }
  selectedPluginIds.value = new Set(selectedPluginIds.value)
  isBatchMode.value = true
}

const clearPluginSelection = () => {
  selectedPluginIds.value.clear()
  selectedPluginIds.value = new Set(selectedPluginIds.value)
  isBatchMode.value = false
  lastClickedPluginIndex.value = -1
}

const selectedPluginCount = computed(() => selectedPluginIds.value.size)

const batchRemovePlugins = async () => {
  const ids = Array.from(selectedPluginIds.value)
  if (ids.length === 0) return
  showBatchDeleteConfirm.value = true
}

const onBatchDeleteConfirm = async () => {
  const ids = Array.from(selectedPluginIds.value)
  try {
    const result = await api.batchRemovePlugins(ids)
    if (result.failed_count > 0) {
      toast.warning(t('common.batchDeleteComplete', { success: result.success_count, failed: result.failed_count }))
    } else {
      toast.success(t('common.batchDeleteSuccess', { count: result.success_count }))
    }
    clearPluginSelection()
    await loadPlugins()
  } catch (error) {
    toast.error(t('common.batchDeleteFailed', { error }))
  }
}

const showBatchDeleteConfirm = ref(false)

const filteredPlugins = computed(() => {
  return plugins.value.filter(plugin => {
    const matchesSearch = searchQuery.value === '' ||
      plugin.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      plugin.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      plugin.author.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesCompatibility = filterCompatibility.value === 'all' ||
      plugin.compatibility === filterCompatibility.value

    const matchesSource = filterSource.value === 'all' ||
      plugin.source.source_type === filterSource.value

    const matchesFavorite = !showFavoritesOnly.value || plugin.is_favorite === true

    return matchesSearch && matchesCompatibility && matchesSource && matchesFavorite
  })
})

const favoritePlugins = computed(() => {
  return plugins.value.filter(p => p.is_favorite).length
})

const loadPlugins = async () => {
  if (hasLoaded.value && pluginStore.plugins.length > 0) {
    return
  }
  isLoading.value = true
  try {
    await pluginStore.loadPlugins()
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLoading.value = false
    hasLoaded.value = true
  }
}

const importFromLocal = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('plugins.selectPluginDir')
    })
    if (selected && typeof selected === 'string') {
      isLoading.value = true
      const result = await api.importPluginFromLocal(selected)
      toast.success(t('common.addProjectSuccess', { name: result.name }))
      await loadPlugins()
    }
  } catch (error) {
    toast.error(t('common.addProjectFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const importFromFile = async () => {
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      title: t('plugins.selectPluginFile'),
      filters: [{ name: 'Plugin Config', extensions: ['cfg'] }]
    })
    if (selected && typeof selected === 'string') {
      isLoading.value = true
      const dirPath = selected.substring(0, selected.lastIndexOf(/[/\\]/.test(selected) ? (selected.includes('\\') ? '\\' : '/') : '/'))
      const result = await api.importPluginFromLocal(dirPath || selected)
      toast.success(t('common.addProjectSuccess', { name: result.name }))
      await loadPlugins()
    }
  } catch (error) {
    toast.error(t('common.addProjectFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const importFromGit = async () => {
  if (!gitUrl.value) {
    toast.warning(t('plugins.enterGitUrl'))
    return
  }
  isLoading.value = true
  try {
    const result = await api.importPluginFromGit(gitUrl.value)
    toast.success(t('common.addProjectSuccess', { name: result.name }))
    gitUrl.value = ''
    showGitDialog.value = false
    await loadPlugins()
  } catch (error) {
    toast.error(t('common.addProjectFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const showAssetLibraryDialog = ref(false)
const assetSearchQuery = ref('')
const assetSearchResults = ref<AssetLibrarySearchResult[]>([])
const isSearchingAssets = ref(false)
const assetCategories = ref<AssetLibraryCategory[]>([])
const assetFilterType = ref<string>('any')
const assetFilterCategory = ref<string>('')
const assetFilterGodotVersion = ref<string>('any')
const assetFilterSupport = ref<string>('')
const assetSortBy = ref<string>('updated')
const assetCurrentPage = ref(0)
const assetTotalPages = ref(0)
const assetTotalItems = ref(0)
const selectedAssetIds = ref<Set<string>>(new Set())
const assetDetail = ref<AssetLibraryAsset | null>(null)
const showAssetDetailDialog = ref(false)
const searchCache = ref<Map<string, { data: AssetLibrarySearchResponse; timestamp: number }>>(new Map())
const categoriesLoaded = ref(false)
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

onMounted(async () => {
  loadPlugins()
  loadTotalStorageStats()
})

onUnmounted(() => {
  // 清理工作
})

const showDeletePluginConfirm = ref(false)
const deletePluginId = ref('')

useDialogEscape(showGitDialog)
useDialogEscape(showPluginDetail)
useDialogEscape(showAssetLibraryDialog)
useDialogEscape(showAssetDetailDialog)
useDialogEscape(showUpdatesDialog)
useDialogEscape(showImportModeDialog)

const openAssetLibrary = async () => {
  showAssetLibraryDialog.value = true
  assetSearchQuery.value = ''
  assetSearchResults.value = []
  selectedAssetIds.value = new Set()
  assetCurrentPage.value = 0
  assetTotalPages.value = 0
  assetTotalItems.value = 0
  if (!categoriesLoaded.value || assetCategories.value.length === 0) {
    try {
      const config = await api.getAssetLibraryConfigure()
      assetCategories.value = config.categories || []
      categoriesLoaded.value = true
    } catch (error) {
      console.error('Failed to load categories:', error)
    }
  }
}

const getCacheKey = () => {
  return JSON.stringify({
    filter: assetSearchQuery.value,
    type: assetFilterType.value,
    category: assetFilterCategory.value,
    support: assetFilterSupport.value,
    godot_version: assetFilterGodotVersion.value,
    sort: assetSortBy.value,
    page: assetCurrentPage.value
  })
}

const searchAssets = (immediate = false) => {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
  }
  if (immediate) {
    doSearch()
  } else {
    searchDebounceTimer = setTimeout(() => {
      doSearch()
    }, 400)
  }
}

const doSearch = async () => {
  isSearchingAssets.value = true
  try {
    const cacheKey = getCacheKey()
    const cached = searchCache.value.get(cacheKey)
    if (cached && Date.now() - cached.timestamp < 5 * 60 * 1000) {
      assetSearchResults.value = cached.data.result
      assetTotalPages.value = cached.data.pages
      assetTotalItems.value = cached.data.total_items
      isSearchingAssets.value = false
      return
    }

    const result = await api.searchAssetLibrary({
      filter: assetSearchQuery.value || undefined,
      type: assetFilterType.value as 'any' | 'addon' | 'project',
      category: assetFilterCategory.value || undefined,
      support: assetFilterSupport.value || undefined,
      godot_version: assetFilterGodotVersion.value !== 'any' ? assetFilterGodotVersion.value : undefined,
      sort: assetSortBy.value as 'rating' | 'cost' | 'name' | 'updated',
      max_results: 20,
      page: assetCurrentPage.value || undefined
    })
    assetSearchResults.value = result.result
    assetTotalPages.value = result.pages
    assetTotalItems.value = result.total_items
    searchCache.value.set(cacheKey, { data: result, timestamp: Date.now() })
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isSearchingAssets.value = false
  }
}

const assetPrevPage = () => {
  if (assetCurrentPage.value > 0) {
    assetCurrentPage.value--
    searchAssets(true)
  }
}

const assetNextPage = () => {
  if (assetCurrentPage.value < assetTotalPages.value - 1) {
    assetCurrentPage.value++
    searchAssets(true)
  }
}

const toggleAssetSelection = (assetId: string) => {
  const newSet = new Set(selectedAssetIds.value)
  if (newSet.has(assetId)) {
    newSet.delete(assetId)
  } else {
    newSet.add(assetId)
  }
  selectedAssetIds.value = newSet
}

const importAsset = async (assetId: string, assetTitle: string) => {
  pluginStore.setImporting(assetId)
  try {
    await api.importFromAssetLibraryWithProgress(assetId)
    toast.success(t('assetLibrary.importSuccess') + ': ' + assetTitle)
    sendImportNotification('Godot Harbor', t('assetLibrary.importSuccess') + ': ' + assetTitle)
    await loadPlugins()
  } catch (error) {
    toast.error(t('assetLibrary.importFailed') + ': ' + error)
  } finally {
    pluginStore.resetImportProgress()
  }
}

const batchImportAssets = async () => {
  const ids = Array.from(selectedAssetIds.value)
  if (ids.length === 0) return

  let successCount = 0
  let failCount = 0
  for (let i = 0; i < ids.length; i++) {
    const assetId = ids[i]
    pluginStore.setImporting(assetId)
    try {
      await api.importFromAssetLibraryWithProgress(assetId)
      successCount++
    } catch {
      failCount++
    } finally {
      if (i === ids.length - 1) {
        pluginStore.resetImportProgress()
      }
    }
  }
  selectedAssetIds.value = new Set()
  if (failCount > 0) {
    toast.warning(t('common.batchDeleteComplete', { success: successCount, failed: failCount }))
    sendImportNotification('Godot Harbor', t('common.batchDeleteComplete', { success: successCount, failed: failCount }))
  } else {
    toast.success(t('common.batchDeleteSuccess', { count: successCount }))
    sendImportNotification('Godot Harbor', t('common.batchImportSuccess', { count: successCount }))
  }
  await loadPlugins()
}

const openAssetDetail = async (assetId: string) => {
  try {
    assetDetail.value = await api.getAssetDetail(assetId)
    showAssetDetailDialog.value = true
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const openPreviewLink = (url: string) => {
  window.open(url, '_blank')
}

const confirmRemovePlugin = (pluginId: string) => {
  deletePluginId.value = pluginId
  showDeletePluginConfirm.value = true
}

const onRemovePluginConfirm = async () => {
  try {
    await api.removePlugin(deletePluginId.value)
    toast.success(t('common.projectDeleted'))
    await loadPlugins()
  } catch (error) {
    toast.error(t('common.deleteFailed', { error }))
  }
}

const toggleFavorite = async (plugin: Plugin) => {
  try {
    const newState = await api.togglePluginFavorite(plugin.plugin_id)
    plugin.is_favorite = newState
    toast.success(newState ? t('plugins.addedToFavorites') : t('plugins.removedFromFavorites'))
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const importFromProjects = async () => {
  showImportModeDialog.value = true
}

const doImportFromProjects = async () => {
  showImportModeDialog.value = false
  isLoading.value = true
  try {
    const importedPlugins = await api.importPluginsFromProjects(importMode.value)
    if (importedPlugins.length > 0) {
      const mode = importMode.value
      const modeLabel = t(`plugins.importMode.${mode}`)
      toast.success(t('plugins.importSuccess', { mode: modeLabel, count: importedPlugins.length }))
    } else {
      toast.info(t('plugins.noNewPluginsFound'))
    }
    await loadPlugins()
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const checkPluginUpdates = async () => {
  isCheckingUpdates.value = true
  try {
    pluginUpdates.value = await api.checkPluginUpdates()
    showUpdatesDialog.value = true
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isCheckingUpdates.value = false
  }
}

const updateGitPlugin = async (pluginId: string) => {
  try {
    const result = await api.updateGitPlugin(pluginId)
    toast.success(t('plugins.updateSuccess', { name: result.name }))
    await loadPlugins()
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const loadTotalStorageStats = async () => {
  try {
    totalStorageStats.value = await api.getTotalStorageStats()
  } catch (e) {
    console.error('Failed to load total storage stats:', e)
  }
}

const cleanupOrphaned = async () => {
  try {
    const count = await api.cleanupOrphanedPluginDirs()
    if (count > 0) {
      toast.success(t('plugins.cleanupOrphaned.success', { count }))
      await loadTotalStorageStats()
    } else {
      toast.info(t('plugins.cleanupOrphaned.noOrphaned'))
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const loadPluginDependencies = async (pluginId: string) => {
  try {
    pluginDependencies.value = await api.resolvePluginDependencies(pluginId)
  } catch (error) {
    console.error('Failed to load dependencies:', error)
    pluginDependencies.value = []
  }
}

const showPluginDetails = async (plugin: Plugin) => {
  selectedPlugin.value = plugin
  showPluginDetail.value = true
  loadPluginDependencies(plugin.plugin_id)
  try {
    pluginStorageStats.value = await api.getPluginStorageStats(plugin.plugin_id)
  } catch (e) {
    console.error('Failed to load storage stats:', e)
    pluginStorageStats.value = null
  }
  try {
    pluginBindings.value = await api.getPluginBindings(plugin.plugin_id)
  } catch (e) {
    console.error('Failed to load plugin bindings:', e)
    pluginBindings.value = []
  }
}

const removePluginVersion = async (pluginId: string, versionId: string) => {
  try {
    await api.removePluginVersion(pluginId, versionId)
    toast.success('版本已删除')
    if (selectedPlugin.value) {
      await showPluginDetails(selectedPlugin.value)
    }
    await loadPlugins()
  } catch (error) {
    toast.error(t('common.deleteFailed', { error }))
  }
}

const closePluginDetail = () => {
  showPluginDetail.value = false
  selectedPlugin.value = null
  pluginDependencies.value = []
  pluginStorageStats.value = null
  pluginBindings.value = []
}

</script>

<template>
  <div class="space-y-6">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('plugins.title') }}</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="checkPluginUpdates"
          :disabled="isCheckingUpdates || isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ isCheckingUpdates ? t('plugins.checkingUpdates') : t('plugins.checkUpdates') }}
        </button>
        <button
          @click="importFromProjects"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('plugins.fromProjects') }}
        </button>
        <button
          @click="importFromLocal"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('plugins.importFromDir') }}
        </button>
        <button
          @click="importFromFile"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('plugins.importFromFile') }}
        </button>
        <button
          @click="openAssetLibrary"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('plugins.assetLibrary') }}
        </button>
        <button
          @click="showGitDialog = true"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('plugins.importFromGit') }}
        </button>
      </div>
    </div>

    <div class="card">
      <div class="flex flex-col lg:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('plugins.search')"
            class="w-full px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary text-sm"
          />
        </div>
        <div class="flex flex-wrap gap-2 items-center">
          <select
            v-model="filterCompatibility"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">{{ t('plugins.allVersions') }}</option>
            <option value="Godot4">Godot 4</option>
            <option value="Godot3">Godot 3</option>
            <option value="Both">通用</option>
          </select>
          <select
            v-model="filterSource"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">{{ t('plugins.allSources') }}</option>
            <option value="Local">{{ t('plugins.localSource') }}</option>
            <option value="Git">Git</option>
            <option value="AssetLibrary">AssetLibrary</option>
          </select>
          <button
            @click="showFavoritesOnly = !showFavoritesOnly"
            :class="[
              'px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              showFavoritesOnly
                ? 'bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-400'
                : 'bg-gray-100 text-gray-700 dark:bg-surface-layer dark:text-content-primary hover:bg-gray-200 dark:hover:bg-surface-layer'
            ]"
          >
            <span class="flex items-center gap-1">
              <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
              </svg>
              {{ favoritePlugins }} 收藏
            </span>
          </button>
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="filteredPlugins.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('plugins.empty') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        从本地目录、文件或 Git 仓库导入插件
      </p>
      <div class="mt-4 flex justify-center gap-3">
        <button
          @click="importFromLocal"
          :disabled="isLoading"
          class="inline-flex items-center gap-1.5 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          从目录导入
        </button>
        <button
          @click="showGitDialog = true"
          :disabled="isLoading"
          class="inline-flex items-center gap-1.5 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
          </svg>
          从 Git 导入
        </button>
      </div>
    </div>

    <div v-else class="space-y-4">
      <div v-if="isBatchMode && selectedPluginCount > 0" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-lg p-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span class="text-sm font-medium text-primary-700 dark:text-primary-300">{{ t('plugins.selectedCount', { count: selectedPluginCount }) }}</span>
          <button
            @click="selectAllPlugins"
            class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
          >
            全选
          </button>
          <button
            @click="clearPluginSelection"
            class="text-xs text-gray-500 dark:text-gray-400 hover:underline"
          >
            取消选择
          </button>
        </div>
        <div class="flex gap-2">
          <button
            @click="batchRemovePlugins"
            class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-lg hover:bg-red-700 transition-colors"
          >
            批量删除 ({{ selectedPluginCount }})
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        <div
          v-for="plugin in filteredPlugins"
          :key="plugin.plugin_id"
          :class="[
            'bg-white dark:bg-surface-card rounded-xl shadow hover:shadow-md transition-shadow p-5',
            selectedPluginIds.has(plugin.plugin_id) ? 'ring-2 ring-primary-500' : ''
          ]"
        >
          <div class="flex items-start justify-between min-w-0">
            <div class="flex items-start gap-3 min-w-0 flex-1">
              <input
                type="checkbox"
                :checked="selectedPluginIds.has(plugin.plugin_id)"
                @click.stop="togglePluginSelection(plugin, $event)"
                class="w-4 h-4 text-primary-600 rounded flex-shrink-0 mt-1 cursor-pointer"
              />
              <div class="min-w-0 flex-1 cursor-pointer" @click="showPluginDetails(plugin)">
                <div class="flex items-center gap-2">
                  <h3 class="text-base font-semibold text-gray-900 dark:text-content-primary truncate">
                    {{ plugin.name }}
                  </h3>
                  <button
                    @click.stop="toggleFavorite(plugin)"
                    :class="[
                      'p-1 rounded transition-colors',
                      plugin.is_favorite
                        ? 'text-yellow-500 hover:text-yellow-600'
                        : 'text-gray-400 dark:text-content-secondary hover:text-yellow-500'
                    ]"
                  >
                    <svg class="w-5 h-5" :fill="plugin.is_favorite ? 'currentColor' : 'none'" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                    </svg>
                  </button>
                </div>
                <p 
                  class="text-sm text-gray-500 dark:text-content-secondary mt-1 line-clamp-2"
                  :title="plugin.description || t('plugins.noDescription')"
                >
                  {{ plugin.description || t('plugins.noDescription') }}
                </p>
              </div>
            </div>
            <button
              @click.stop="confirmRemovePlugin(plugin.plugin_id)"
              class="text-red-600 hover:text-red-800 ml-2"
            >
              <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          <div class="mt-3 flex items-center justify-between text-sm text-gray-600 dark:text-content-secondary">
            <span>v{{ plugin.versions[0]?.version || '1.0.0' }}</span>
            <span>{{ plugin.author || t('plugins.unknownAuthor') }}</span>
          </div>
          <div class="mt-2 flex items-center gap-2 flex-wrap">
            <span class="badge badge-neutral">
              {{ plugin.compatibility === 'Godot4' ? 'Godot 4' : plugin.compatibility === 'Godot3' ? 'Godot 3' : plugin.compatibility === 'Both' ? t('plugins.generic') : t('plugins.unknown') }}
            </span>
            <span class="badge badge-neutral">
              {{ plugin.source.source_type === 'Local' ? t('plugins.localSource') : plugin.source.source_type === 'Git' ? 'Git' : 'AssetLibrary' }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showGitDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showGitDialog = false; gitUrl = ''">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('plugins.importFromGit') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">
          输入 Git 仓库 URL，将克隆并导入其中的 Godot 插件
        </p>
        <input
          v-model="gitUrl"
          type="text"
          placeholder="https://github.com/user/plugin-repo.git"
          class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
        />
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showGitDialog = false; gitUrl = ''"
            class="btn-secondary"
          >
            取消
          </button>
          <button
            @click="importFromGit"
            :disabled="isLoading || !gitUrl"
            class="btn-primary disabled:opacity-50"
          >
            导入
          </button>
        </div>
      </div>
    </div>

    <div v-if="showPluginDetail && selectedPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="closePluginDetail">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex items-center justify-between mb-2">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
            {{ selectedPlugin.name }}
          </h3>
          <button @click="closePluginDetail" class="text-gray-500 dark:text-content-secondary hover:text-gray-700 dark:hover:text-content-primary">
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="mb-4 flex items-center gap-3 flex-wrap text-sm text-gray-500 dark:text-content-secondary">
          <span>{{ t('plugins.author') }}: {{ selectedPlugin.author || t('plugins.unknownAuthor') }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ selectedPlugin.compatibility === 'Godot4' ? 'Godot 4' : selectedPlugin.compatibility === 'Godot3' ? 'Godot 3' : selectedPlugin.compatibility === 'Both' ? t('plugins.generic') : t('plugins.unknown') }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ selectedPlugin.source.source_type === 'Local' ? t('plugins.localSource') : selectedPlugin.source.source_type === 'Git' ? 'Git' : 'AssetLibrary' }}</span>
          <span v-if="pluginStorageStats" class="text-gray-300 dark:text-content-secondary">|</span>
          <span v-if="pluginStorageStats">{{ pluginStorageStats.total_size_display }}</span>
        </div>

        <div class="flex-1 overflow-y-auto space-y-4">
          <div>
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('plugins.description') }}</h4>
            <p class="text-sm text-gray-600 dark:text-content-secondary whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              {{ selectedPlugin.description || t('plugins.noDescription') }}
            </p>
          </div>

          <div>
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">
              版本列表 ({{ selectedPlugin.versions.length }})
            </h4>
            <div class="space-y-2 bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              <div v-for="version in selectedPlugin.versions" :key="version.version_id"
                class="flex items-center justify-between py-1.5 border-b border-gray-200 dark:border-gray-600 last:border-0">
                <div>
                  <span class="text-sm font-medium text-gray-900 dark:text-content-primary">v{{ version.version }}</span>
                  <span class="text-xs text-gray-500 dark:text-content-secondary ml-2">
                    {{ new Date(version.created_at).toLocaleDateString() }}
                  </span>
                  <span class="text-xs text-gray-400 dark:text-content-secondary ml-2">
                    {{ version.units.length }} 个单元
                  </span>
                </div>
                <button
                  v-if="selectedPlugin.versions.length > 1"
                  @click="removePluginVersion(selectedPlugin.plugin_id, version.version_id)"
                  class="text-xs text-red-500 hover:text-red-700"
                >
                  删除
                </button>
              </div>
            </div>
          </div>

          <div v-if="pluginBindings.length > 0">
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">
              挂载到 ({{ pluginBindings.length }} 个项目)
            </h4>
            <div class="space-y-1 bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              <div v-for="binding in pluginBindings" :key="binding.project_id + binding.mount_path"
                class="text-sm text-gray-600 dark:text-content-secondary py-1">
                <span class="font-mono text-xs">{{ binding.mount_path }}</span>
              </div>
            </div>
          </div>

          <div v-if="pluginDependencies.length > 0">
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">依赖关系</h4>
            <div class="space-y-2 bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              <div v-for="dep in pluginDependencies" :key="dep.plugin_id" class="text-sm text-gray-600 dark:text-content-secondary">
                <span class="font-medium">{{ dep.plugin_id }}</span>
                <span v-if="dep.version_constraint"> ({{ dep.version_constraint }})</span>
                <span v-if="dep.is_optional" class="ml-2 text-xs text-gray-500 dark:text-content-secondary">(可选)</span>
              </div>
            </div>
          </div>

          <div>
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">来源</h4>
            <p class="text-sm text-gray-600 dark:text-content-secondary bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              {{ selectedPlugin.source.source_type === 'Local' ? '本地目录' : selectedPlugin.source.source_type === 'Git' ? 'Git 仓库' : 'AssetLibrary' }}
              <span v-if="selectedPlugin.source.url" class="block text-xs mt-1 break-all font-mono">{{ selectedPlugin.source.url }}</span>
            </p>
          </div>

          <div v-if="pluginStorageStats" class="grid grid-cols-3 gap-3">
            <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 text-center">
              <div class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.version_count }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">版本</div>
            </div>
            <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 text-center">
              <div class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.binding_count }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">挂载</div>
            </div>
            <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 text-center">
              <div class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.total_size_display }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">存储</div>
            </div>
          </div>
        </div>

        <div class="flex justify-end mt-4 pt-3 border-t border-gray-200 dark:border-gray-700">
          <button
            @click="closePluginDetail"
            class="btn-secondary"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

    <div v-if="showAssetLibraryDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showAssetLibraryDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-2xl shadow-xl max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('assetLibrary.title') }}</h3>
          <button @click="showAssetLibraryDialog = false" class="text-gray-500 dark:text-content-secondary hover:text-gray-700 dark:hover:text-content-primary">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="flex gap-2 mb-3">
          <input
            v-model="assetSearchQuery"
            type="text"
            :placeholder="t('assetLibrary.searchPlaceholder')"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
            @input="searchAssets()"
            @keyup.enter="searchAssets(true)"
          />
          <button
            @click="searchAssets(true)"
            :disabled="isSearchingAssets"
            class="btn-primary disabled:opacity-50 text-sm"
          >
            {{ isSearchingAssets ? t('assetLibrary.searching') : t('assetLibrary.search') }}
          </button>
        </div>

        <div class="flex flex-wrap gap-2 mb-3">
          <select v-model="assetFilterType" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
            <option value="any">{{ t('assetLibrary.typeAny') }}</option>
            <option value="addon">{{ t('assetLibrary.typeAddon') }}</option>
            <option value="project">{{ t('assetLibrary.typeProject') }}</option>
          </select>
          <select v-model="assetFilterCategory" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
            <option value="">{{ t('assetLibrary.categoryAll') }}</option>
            <option v-for="cat in assetCategories" :key="cat.id" :value="cat.id">{{ cat.name }}</option>
          </select>
          <select v-model="assetFilterGodotVersion" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
            <option value="any">{{ t('assetLibrary.godotVersion') }}: Any</option>
            <option value="4.0">Godot 4.x</option>
            <option value="3.0">Godot 3.x</option>
          </select>
          <select v-model="assetFilterSupport" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
            <option value="">{{ t('assetLibrary.supportLevel') }}: All</option>
            <option value="official">{{ t('assetLibrary.supportOfficial') }}</option>
            <option value="featured">{{ t('assetLibrary.supportFeatured') }}</option>
            <option value="community">{{ t('assetLibrary.supportCommunity') }}</option>
            <option value="testing">{{ t('assetLibrary.supportTesting') }}</option>
          </select>
          <select v-model="assetSortBy" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
            <option value="updated">{{ t('assetLibrary.sortUpdated') }}</option>
            <option value="rating">{{ t('assetLibrary.sortRating') }}</option>
            <option value="name">{{ t('assetLibrary.sortName') }}</option>
            <option value="cost">{{ t('assetLibrary.sortCost') }}</option>
          </select>
        </div>

        <div v-if="selectedAssetIds.size > 0" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-lg p-2 mb-3 flex items-center justify-between">
          <span class="text-xs font-medium text-primary-700 dark:text-primary-300">{{ t('assetLibrary.selectedCount', { count: selectedAssetIds.size }) }}</span>
          <button
            @click="batchImportAssets"
            :disabled="!!pluginStore.isImporting"
            class="px-3 py-1 bg-primary-600 text-white text-xs rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ t('assetLibrary.batchImport') }} ({{ selectedAssetIds.size }})
          </button>
        </div>

        <div v-if="pluginStore.importProgress && pluginStore.isImporting" class="mb-3">
          <div class="flex items-center justify-between text-xs text-gray-600 dark:text-content-secondary mb-1">
            <span>{{ pluginStore.importProgress.message }}</span>
            <span>{{ Math.round(pluginStore.importProgress.progress * 100) }}%</span>
          </div>
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              class="bg-primary-600 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${pluginStore.importProgress.progress * 100}%` }"
            ></div>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto space-y-2">
          <div v-if="assetSearchResults.length === 0 && !isSearchingAssets" class="text-center py-8 text-gray-500 dark:text-content-secondary">
            {{ t('assetLibrary.noResults') }}
          </div>
          <div v-if="isSearchingAssets" class="flex justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
          </div>
          <div
            v-for="asset in assetSearchResults"
            :key="asset.asset_id"
            :class="[
              'bg-gray-50 dark:bg-surface-layer rounded-lg p-3 transition-colors',
              selectedAssetIds.has(asset.asset_id) ? 'ring-2 ring-primary-500' : ''
            ]"
          >
            <div class="flex items-center gap-3">
              <input
                type="checkbox"
                :checked="selectedAssetIds.has(asset.asset_id)"
                @change="toggleAssetSelection(asset.asset_id)"
                class="w-4 h-4 text-primary-600 rounded flex-shrink-0 cursor-pointer"
              />
              <img
                v-if="asset.icon_url"
                :src="asset.icon_url"
                :alt="asset.title"
                class="w-10 h-10 rounded object-cover flex-shrink-0"
                loading="lazy"
                @error="($event.target as HTMLImageElement).style.display = 'none'"
              />
              <div v-else class="w-10 h-10 rounded bg-gray-200 dark:bg-gray-600 flex items-center justify-center flex-shrink-0">
                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                </svg>
              </div>
              <div class="flex-1 min-w-0 cursor-pointer" @click="openAssetDetail(asset.asset_id)">
                <div class="flex items-center gap-2">
                  <span class="font-medium text-gray-900 dark:text-content-primary text-sm truncate">{{ asset.title }}</span>
                  <span v-if="asset.support_level === 'official'" class="px-1.5 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400">{{ t('assetLibrary.supportOfficial') }}</span>
                  <span v-else-if="asset.support_level === 'featured'" class="px-1.5 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400">{{ t('assetLibrary.supportFeatured') }}</span>
                </div>
                <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">
                  {{ asset.author }} · {{ asset.category }} · {{ asset.cost }}
                </div>
              </div>
              <button
                @click="importAsset(asset.asset_id, asset.title)"
                :disabled="pluginStore.isImporting === asset.asset_id"
                class="btn-primary disabled:opacity-50 text-xs px-3 py-1.5 flex-shrink-0"
              >
                {{ pluginStore.isImporting === asset.asset_id ? t('assetLibrary.importing') : t('assetLibrary.import') }}
              </button>
            </div>
          </div>
        </div>

        <div v-if="assetTotalPages > 0" class="flex items-center justify-between mt-4 pt-3 border-t border-gray-200 dark:border-gray-700">
          <span class="text-xs text-gray-500 dark:text-content-secondary">
            {{ t('assetLibrary.totalItems', { count: assetTotalItems }) }}
          </span>
          <div class="flex items-center gap-2">
            <button
              @click="assetPrevPage"
              :disabled="assetCurrentPage === 0"
              class="px-3 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-surface-layer text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-card disabled:opacity-50"
            >
              {{ t('assetLibrary.prevPage') }}
            </button>
            <span class="text-xs text-gray-600 dark:text-content-secondary">
              {{ t('assetLibrary.page', { current: assetCurrentPage + 1, total: assetTotalPages }) }}
            </span>
            <button
              @click="assetNextPage"
              :disabled="assetCurrentPage >= assetTotalPages - 1"
              class="px-3 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-surface-layer text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-card disabled:opacity-50"
            >
              {{ t('assetLibrary.nextPage') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showAssetDetailDialog && assetDetail" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showAssetDetailDialog = false; assetDetail = null">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ assetDetail.title }}</h3>
          <button @click="showAssetDetailDialog = false; assetDetail = null" class="text-gray-500 dark:text-content-secondary hover:text-gray-700 dark:hover:text-content-primary">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="flex items-center gap-3 mb-4">
          <img
            v-if="assetDetail.icon_url"
            :src="assetDetail.icon_url"
            :alt="assetDetail.title"
            class="w-12 h-12 rounded object-cover"
          />
          <div>
            <div class="text-sm text-gray-600 dark:text-content-secondary">{{ t('assetLibrary.author') }}: {{ assetDetail.author }}</div>
            <div class="text-sm text-gray-600 dark:text-content-secondary">{{ t('assetLibrary.license') }}: {{ assetDetail.cost }}</div>
            <div class="text-sm text-gray-600 dark:text-content-secondary">{{ t('assetLibrary.rating') }}: {{ assetDetail.rating }}/5</div>
          </div>
        </div>

        <div v-if="assetDetail.previews && assetDetail.previews.length > 0" class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('assetLibrary.previews') }}</h4>
          <div class="flex gap-2 overflow-x-auto pb-2">
            <img
              v-for="preview in assetDetail.previews.filter(p => p.type === 'image')"
              :key="preview.preview_id"
              :src="preview.thumbnail"
              class="h-20 rounded object-cover flex-shrink-0 cursor-pointer hover:opacity-80"
              loading="lazy"
              @click="openPreviewLink(preview.link)"
            />
          </div>
        </div>

        <div class="flex-1 overflow-y-auto mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('assetLibrary.description') }}</h4>
          <p class="text-sm text-gray-600 dark:text-content-secondary whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            {{ assetDetail.description || t('assetLibrary.noDescription') }}
          </p>
        </div>

        <div class="flex items-center gap-2">
          <button
            @click="importAsset(assetDetail.asset_id, assetDetail.title); showAssetDetailDialog = false; assetDetail = null"
            :disabled="pluginStore.isImporting === assetDetail.asset_id"
            class="btn-primary disabled:opacity-50 text-sm"
          >
            {{ pluginStore.isImporting === assetDetail.asset_id ? t('assetLibrary.importing') : t('assetLibrary.import') }}
          </button>
          <a
            v-if="assetDetail.browse_url"
            :href="assetDetail.browse_url"
            target="_blank"
            class="px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-card"
          >
            {{ t('assetLibrary.detail') }}
          </a>
          <div class="flex-1"></div>
          <button
            @click="showAssetDetailDialog = false; assetDetail = null"
            class="btn-secondary text-sm"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showUpdatesDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showUpdatesDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-lg shadow-xl" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">插件更新检查</h3>
          <button @click="showUpdatesDialog = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="space-y-3 max-h-80 overflow-y-auto">
          <div v-if="pluginUpdates.length === 0" class="text-center py-8 text-gray-500 dark:text-gray-400">
            没有可检查更新的插件
          </div>
          <div v-for="update in pluginUpdates" :key="update.plugin_id" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <span class="font-medium text-gray-900 dark:text-gray-100">{{ update.plugin_id }}</span>
                <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                  当前版本: {{ update.current_version }} → 最新版本: {{ update.latest_version }}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button
                  v-if="update.update_available"
                  @click="updateGitPlugin(update.plugin_id)"
                  class="px-3 py-1 bg-primary-600 text-white text-xs rounded-lg hover:bg-primary-700"
                >
                  更新
                </button>
                <span v-if="update.update_available" class="px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  有更新
                </span>
                <span v-else class="px-2 py-1 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-600 dark:text-gray-400">
                  已是最新
                </span>
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-end mt-4">
          <button
            @click="showUpdatesDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

    <div v-if="showImportModeDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showImportModeDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('plugins.importFromProject.title') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">{{ t('plugins.importFromProject.modeSelect') }}</p>
        <div class="space-y-3 mb-6">
          <label class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors"
            :class="importMode === 'copy' ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-surface-layer'">
            <input type="radio" v-model="importMode" value="copy" class="mt-1" />
            <div>
              <div class="font-medium text-gray-900 dark:text-content-primary text-sm">{{ t('plugins.importModes.copy.label') }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">{{ t('plugins.importModes.copy.desc') }}</div>
            </div>
          </label>
          <label class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors"
            :class="importMode === 'move' ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-surface-layer'">
            <input type="radio" v-model="importMode" value="move" class="mt-1" />
            <div>
              <div class="font-medium text-gray-900 dark:text-content-primary text-sm">{{ t('plugins.importModes.move.label') }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">{{ t('plugins.importModes.move.desc') }}</div>
            </div>
          </label>
          <label class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors"
            :class="importMode === 'reference' ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-surface-layer'">
            <input type="radio" v-model="importMode" value="reference" class="mt-1" />
            <div>
              <div class="font-medium text-gray-900 dark:text-content-primary text-sm">{{ t('plugins.importModes.reference.label') }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">{{ t('plugins.importModes.reference.desc') }}</div>
            </div>
          </label>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showImportModeDialog = false" class="btn-secondary">{{ t('plugins.importFromProject.cancel') }}</button>
          <button @click="doImportFromProjects" class="btn-primary">{{ t('plugins.importFromProject.startImport') }}</button>
        </div>
      </div>
    </div>

    <div v-if="totalStorageStats" class="card">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4 text-sm text-gray-600 dark:text-content-secondary">
          <span>{{ totalStorageStats.total_plugins }} 个插件</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ totalStorageStats.total_versions }} 个版本</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ totalStorageStats.total_bindings }} 个挂载</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>占用 {{ totalStorageStats.total_size_display }}</span>
          <span v-if="totalStorageStats.orphaned_size_bytes > 0" class="text-orange-500">
            | {{ totalStorageStats.orphaned_size_display }} 可清理
          </span>
          <span v-if="totalStorageStats.duplicate_hash_count > 0" class="text-yellow-500">
            | {{ totalStorageStats.duplicate_hash_count }} 组内容重复
          </span>
        </div>
        <button
          v-if="totalStorageStats.orphaned_size_bytes > 0"
          @click="cleanupOrphaned"
          class="px-3 py-1 text-xs border border-orange-300 dark:border-orange-700 text-orange-600 dark:text-orange-400 rounded-lg hover:bg-orange-50 dark:hover:bg-orange-900/20"
        >
          清理孤立文件
        </button>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeletePluginConfirm"
      title="确认删除插件"
      description="此操作将从仓库中移除插件，但不会影响已挂载到项目中的副本。"
      confirm-text="确认删除"
      @confirm="onRemovePluginConfirm"
    />

    <ConfirmDialog
      v-model="showBatchDeleteConfirm"
      title="批量删除插件"
      :description="`确定要删除选中的 ${selectedPluginCount} 个插件吗？此操作将从仓库中移除插件，但不会影响已挂载到项目中的副本。`"
      confirm-text="确认批量删除"
      @confirm="onBatchDeleteConfirm"
    />
  </div>
</template>