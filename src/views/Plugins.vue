<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { api } from '@/api'
import type { Plugin, Project, PluginUpdateInfo, PluginDependency, AssetLibrarySearchResult, AssetLibrarySearchResponse, AssetLibraryCategory, AssetLibraryAsset, PluginStorageStats, ProjectBinding, TotalStorageStats, DuplicateCheckResult, ScannedPlugin } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { usePluginStore } from '@/stores'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const pluginStore = usePluginStore()
const route = useRoute()

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

const showAddMenu = ref(false)

const activeTab = ref<'repository' | 'bindings'>('repository')

const linkerProjects = ref<Project[]>([])
const linkerBindings = ref<ProjectBinding[]>([])
const selectedLinkId = ref<string | null>(null)
const selectedLinkProjectIds = ref<Set<string>>(new Set())
const selectedLinkPluginIds = ref<Set<string>>(new Set())
const showLinkerApplyDialog = ref(false)
const showLinkerBatchBindDialog = ref(false)
const showLinkerBatchUnbindDialog = ref(false)
const showLinkerBatchApplyDialog = ref(false)
const showLinkerVersionSelect = ref(false)
const versionSelectPlugin = ref<Plugin | null>(null)
const selectedVersionIdx = ref(0)
const selectedUnitIdx = ref(0)
const isLinkerApplying = ref(false)
const isLinkerBatchBinding = ref(false)
const isLinkerBatchUnbinding = ref(false)
const isLinkerBatchApplying = ref(false)
const linkerApplyResult = ref<any>(null)
const showLinkerApplyResult = ref(false)
const showLinkerBatchApplyResult = ref(false)
const batchApplyResults = ref<any[]>([])
const showGraphView = ref(false)
const linkerHasPendingChanges = ref(false)
const linkerSearchQuery = ref('')

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

const loadPlugins = async (force = false) => {
  if (!force && hasLoaded.value && pluginStore.plugins.length > 0) {
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
      try {
        const dupCheck = await api.checkPluginDuplicate(selected)
        if (dupCheck.is_duplicate) {
          isLoading.value = false
          duplicateCheckResult.value = dupCheck
          pendingImportAction.value = async () => {
            isLoading.value = true
            try {
              const result = await api.importPluginFromLocal(selected)
              toast.success(t('plugins.importPluginSuccess', { name: result.name }))
              await loadPlugins(true)
            } catch (error) {
              toast.error(t('common.addProjectFailed', { error }))
            } finally {
              isLoading.value = false
            }
          }
          showDuplicateConfirm.value = true
          return
        }
      } catch {
        // duplicate check failed, proceed with import
      }
      isLoading.value = true
      const result = await api.importPluginFromLocal(selected)
      toast.success(t('plugins.importPluginSuccess', { name: result.name }))
      await loadPlugins(true)
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
      toast.success(t('plugins.importPluginSuccess', { name: result.name }))
      await loadPlugins(true)
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
    toast.success(t('plugins.importPluginSuccess', { name: result.name }))
    gitUrl.value = ''
    showGitDialog.value = false
    await loadPlugins(true)
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

const addMenuRef = ref<HTMLElement | null>(null)

const handleClickOutside = (event: MouseEvent) => {
  if (showAddMenu.value && addMenuRef.value && !addMenuRef.value.contains(event.target as Node)) {
    showAddMenu.value = false
  }
}

onMounted(async () => {
  loadPlugins()
  loadTotalStorageStats()
  document.addEventListener('click', handleClickOutside)

  if (route.query.tab === 'bindings') {
    activeTab.value = 'bindings'
    await loadLinkerData()
    if (route.query.project && typeof route.query.project === 'string') {
      selectedLinkId.value = route.query.project
      selectedLinkProjectIds.value = new Set([route.query.project])
      await loadLinkerBindings(route.query.project)
    }
  }
})

onUnmounted(() => {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
    searchDebounceTimer = null
  }
  searchCache.value.clear()
  document.removeEventListener('click', handleClickOutside)
})

const showDeletePluginConfirm = ref(false)
const deletePluginId = ref('')

const showDuplicateConfirm = ref(false)
const duplicateCheckResult = ref<DuplicateCheckResult | null>(null)
const pendingImportAction = ref<(() => Promise<void>) | null>(null)

useDialogEscape(showGitDialog)
useDialogEscape(showPluginDetail)
useDialogEscape(showAssetLibraryDialog)
useDialogEscape(showAssetDetailDialog)
useDialogEscape(showUpdatesDialog)
useDialogEscape(showImportModeDialog)
useDialogEscape(showDuplicateConfirm)

const showBindDialog = ref(false)
const bindTargetPlugin = ref<Plugin | null>(null)
const bindProjects = ref<Project[]>([])
const bindSelectedProjectIds = ref<Set<string>>(new Set())
const bindSelectedVersionIdx = ref(0)
const bindSelectedUnitIdx = ref(0)
const isBinding = ref(false)

useDialogEscape(showBindDialog)

const openBindDialog = async (plugin: Plugin) => {
  bindTargetPlugin.value = plugin
  bindSelectedVersionIdx.value = 0
  bindSelectedUnitIdx.value = 0
  bindSelectedProjectIds.value = new Set()
  try {
    bindProjects.value = await api.getProjects()
  } catch (e) {
    bindProjects.value = []
  }
  showBindDialog.value = true
}

const confirmBind = async (applyNow = false) => {
  if (!bindTargetPlugin.value || bindSelectedProjectIds.value.size === 0) return
  const plugin = bindTargetPlugin.value
  const version = plugin.versions[bindSelectedVersionIdx.value]
  const unit = version?.units[bindSelectedUnitIdx.value]
  if (!version || !unit) {
    toast.warning(t('plugins.bindDialog.noUnits'))
    return
  }
  isBinding.value = true
  const projectIds = Array.from(bindSelectedProjectIds.value)
  let successCount = 0
  let failCount = 0
  for (const projectId of projectIds) {
    try {
      await api.bindPlugin(projectId, plugin.plugin_id, version.version_id, unit.unit_id, unit.subdirectory || `addons/${unit.name}`)
      successCount++
    } catch {
      failCount++
    }
  }
  if (applyNow && successCount > 0) {
    for (const projectId of projectIds) {
      try {
        await api.applyChanges(projectId)
      } catch {
        // ignore apply errors for individual projects
      }
    }
  }
  isBinding.value = false
  showBindDialog.value = false
  if (failCount > 0) {
    toast.warning(t('plugins.bindDialog.partialSuccess', { success: successCount, failed: failCount }))
  } else if (applyNow) {
    toast.success(t('plugins.bindDialog.bindAndApplySuccess', { count: successCount, name: plugin.name }))
  } else {
    toast.success(t('plugins.bindDialog.success', { count: successCount, name: plugin.name }))
  }
  bindTargetPlugin.value = null
}

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
    toast.success(t('plugins.importPluginSuccess', { name: assetTitle }))
    sendImportNotification('Godot Harbor', t('plugins.importPluginSuccess', { name: assetTitle }))
    await loadPlugins(true)
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
    toast.warning(t('plugins.depDialog.partialSuccess', { success: successCount, failed: failCount }))
    sendImportNotification('Godot Harbor', t('plugins.depDialog.partialSuccess', { success: successCount, failed: failCount }))
  } else {
    toast.success(t('common.batchImportSuccess', { count: successCount }))
    sendImportNotification('Godot Harbor', t('common.batchImportSuccess', { count: successCount }))
  }
  await loadPlugins(true)
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

const deletePluginBindings = ref<ProjectBinding[]>([])
const deletePluginName = ref('')
const deletePluginProjects = ref<Map<string, string>>(new Map())

const confirmRemovePlugin = async (pluginId: string) => {
  deletePluginId.value = pluginId
  const plugin = plugins.value.find(p => p.plugin_id === pluginId)
  deletePluginName.value = plugin?.name || ''
  try {
    deletePluginBindings.value = await api.getPluginBindings(pluginId)
  } catch {
    deletePluginBindings.value = []
  }
  try {
    const projects = await api.getProjects()
    const map = new Map<string, string>()
    for (const p of projects) {
      map.set(p.project_id, p.name)
    }
    deletePluginProjects.value = map
  } catch {
    deletePluginProjects.value = new Map()
  }
  showDeletePluginConfirm.value = true
}

const onRemovePluginConfirm = async () => {
  try {
    if (deletePluginBindings.value.length > 0) {
      for (const binding of deletePluginBindings.value) {
        try {
          await api.unbindPlugin(binding.project_id, deletePluginId.value)
        } catch {
          // ignore individual unbind errors
        }
      }
    }
    await api.removePlugin(deletePluginId.value)
    toast.success(t('common.projectDeleted'))
    await loadPlugins(true)
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

const scannedPlugins = ref<ScannedPlugin[]>([])
const isScanningProjects = ref(false)
const showScanPreviewDialog = ref(false)

useDialogEscape(showScanPreviewDialog)

const importFromProjects = async () => {
  isScanningProjects.value = true
  try {
    const result = await api.scanProjectPlugins()
    if (result.length === 0) {
      toast.info(t('plugins.noNewPluginsFound'))
      isScanningProjects.value = false
      return
    }
    scannedPlugins.value = result
    showScanPreviewDialog.value = true
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isScanningProjects.value = false
  }
}

const startImportFromPreview = () => {
  showScanPreviewDialog.value = false
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
    await loadPlugins(true)
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

const expandedReleaseNotes = ref<Set<string>>(new Set())

const updatablePluginIds = computed(() =>
  pluginUpdates.value.filter(u => u.update_available).map(u => u.plugin_id)
)

const isBatchUpdating = ref(false)

const batchUpdatePlugins = async () => {
  isBatchUpdating.value = true
  let successCount = 0
  let failCount = 0
  for (const pluginId of updatablePluginIds.value) {
    try {
      await api.updateGitPlugin(pluginId)
      successCount++
    } catch {
      failCount++
    }
  }
  isBatchUpdating.value = false
  if (failCount > 0) {
    toast.warning(t('plugins.updateCheck.batchPartial', { success: successCount, failed: failCount }))
  } else {
    toast.success(t('plugins.updateCheck.batchSuccess', { count: successCount }))
  }
  await loadPlugins()
  pluginUpdates.value = await api.checkPluginUpdates()
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

const bindingProjects = ref<Map<string, string>>(new Map())

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
  try {
    const projects = await api.getProjects()
    const map = new Map<string, string>()
    for (const p of projects) {
      map.set(p.project_id, p.name)
    }
    bindingProjects.value = map
  } catch {
    bindingProjects.value = new Map()
  }
}

const removePluginVersion = async (pluginId: string, versionId: string) => {
  try {
    await api.removePluginVersion(pluginId, versionId)
    toast.success(t('plugins.versionDeleted'))
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

const repairBinding = async (projectId: string, pluginId: string) => {
  try {
    await api.repairBinding(projectId, pluginId)
    toast.success(t('plugins.bindDialog.repairSuccess'))
    if (selectedPlugin.value) {
      pluginBindings.value = await api.getPluginBindings(selectedPlugin.value.plugin_id)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const installedPluginIds = computed(() => new Set(plugins.value.map(p => p.plugin_id)))

const importedAssetIds = computed(() => {
  const ids = new Set<string>()
  for (const p of plugins.value) {
    if (p.source.source_type === 'AssetLibrary' && p.source.url) {
      const match = p.source.url.match(/^asset-library:\/\/(\d+)$/)
      if (match) ids.add(match[1])
    }
  }
  return ids
})

const missingDepPluginIds = computed(() => {
  return pluginDependencies.value
    .filter(d => !d.is_optional && !installedPluginIds.value.has(d.plugin_id))
    .map(d => d.plugin_id)
})

const isInstallingDeps = ref(false)

const installMissingDeps = async () => {
  if (missingDepPluginIds.value.length === 0) return
  isInstallingDeps.value = true
  let successCount = 0
  let failCount = 0
  for (const depId of missingDepPluginIds.value) {
    try {
      const dep = pluginDependencies.value.find(d => d.plugin_id === depId)
      if (dep?.version_constraint) {
        try {
          await api.importPluginFromGit(dep.version_constraint)
        } catch {
          // version_constraint may not be a valid git URL, skip
        }
      }
      successCount++
    } catch {
      failCount++
    }
  }
  isInstallingDeps.value = false
  if (failCount > 0) {
    toast.warning(t('plugins.depDialog.partialSuccess', { success: successCount, failed: failCount }))
  } else {
    toast.success(t('plugins.depDialog.success', { count: successCount }))
  }
  await loadPlugins(true)
  if (selectedPlugin.value) {
    pluginDependencies.value = await api.resolvePluginDependencies(selectedPlugin.value.plugin_id)
  }
}

const goToBindings = async (plugin: Plugin) => {
  activeTab.value = 'bindings'
  await loadLinkerData()
  selectedLinkPluginIds.value = new Set([plugin.plugin_id])
}

const loadLinkerData = async () => {
  try {
    const [projs] = await Promise.all([
      api.getProjects(),
      api.getPlugins()
    ])
    linkerProjects.value = projs
    if (!hasLoaded.value) {
      await loadPlugins(true)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const loadLinkerBindings = async (projectId: string) => {
  try {
    linkerBindings.value = await api.getProjectBindings(projectId)
  } catch (error) {
    linkerBindings.value = []
  }
}

const selectLinkProject = async (project: Project, event: MouseEvent) => {
  if (event.ctrlKey || event.metaKey) {
    if (selectedLinkProjectIds.value.has(project.project_id)) {
      selectedLinkProjectIds.value.delete(project.project_id)
    } else {
      selectedLinkProjectIds.value.add(project.project_id)
    }
    selectedLinkProjectIds.value = new Set(selectedLinkProjectIds.value)
  } else {
    selectedLinkId.value = project.project_id
    selectedLinkProjectIds.value = new Set([project.project_id])
    await loadLinkerBindings(project.project_id)
  }
}

const bindPluginToProject = async (plugin: Plugin) => {
  if (selectedLinkProjectIds.value.size === 0) {
    toast.warning(t('linker.selectProject'))
    return
  }
  if (plugin.versions.length === 0) {
    toast.warning(t('linker.noPluginVersions'))
    return
  }
  if (plugin.versions.length > 1 || plugin.versions[0]?.units.length > 1) {
    versionSelectPlugin.value = plugin
    selectedVersionIdx.value = 0
    selectedUnitIdx.value = 0
    showLinkerVersionSelect.value = true
    return
  }
  const version = plugin.versions[0]
  const unit = version.units[0]
  if (!unit) {
    toast.warning(t('linker.noPluginUnits'))
    return
  }
  await doBindPlugin(plugin, version, unit)
}

const doBindPlugin = async (plugin: Plugin, version: any, unit: any) => {
  const mountPath = unit.subdirectory || `addons/${unit.name}`
  for (const projectId of selectedLinkProjectIds.value) {
    const existingBinding = linkerBindings.value.find(
      b => b.project_id === projectId && b.mount_path === mountPath
    )
    if (existingBinding) {
      const conflictPlugin = plugins.value.find(p => p.plugin_id === existingBinding.plugin_id)
      toast.warning(t('linker.mountConflict', { path: mountPath, plugin: conflictPlugin?.name || existingBinding.plugin_id }))
      return
    }
  }
  try {
    for (const projectId of selectedLinkProjectIds.value) {
      await api.bindPlugin(projectId, plugin.plugin_id, version.version_id, unit.unit_id, mountPath)
    }
    toast.success(t('linker.pluginBound', { name: plugin.name, version: version.version }))
    linkerHasPendingChanges.value = true
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const confirmVersionSelect = async () => {
  if (!versionSelectPlugin.value) return
  const plugin = versionSelectPlugin.value
  const version = plugin.versions[selectedVersionIdx.value]
  const unit = version?.units[selectedUnitIdx.value]
  if (!version || !unit) return
  showLinkerVersionSelect.value = false
  await doBindPlugin(plugin, version, unit)
}

const unbindPluginFromProject = async (binding: ProjectBinding) => {
  try {
    await api.unbindPlugin(binding.project_id, binding.plugin_id)
    toast.success(t('linker.pluginUnbound'))
    linkerHasPendingChanges.value = true
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const confirmLinkerApply = async () => {
  if (!selectedLinkId.value) return
  isLinkerApplying.value = true
  try {
    linkerApplyResult.value = await api.applyChanges(selectedLinkId.value)
    showLinkerApplyResult.value = true
    linkerHasPendingChanges.value = false
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLinkerApplying.value = false
  }
}

const batchBindPlugins = async () => {
  if (selectedLinkProjectIds.value.size === 0) {
    toast.warning(t('linker.selectProject'))
    return
  }
  if (selectedLinkPluginIds.value.size === 0) {
    toast.warning(t('linker.selectPlugin'))
    return
  }
  showLinkerBatchBindDialog.value = true
}

const confirmBatchBind = async () => {
  isLinkerBatchBinding.value = true
  const requests = []
  for (const projectId of selectedLinkProjectIds.value) {
    for (const pluginId of selectedLinkPluginIds.value) {
      const plugin = plugins.value.find(p => p.plugin_id === pluginId)
      if (plugin && plugin.versions.length > 0) {
        const version = plugin.versions[0]
        const unit = version.units[0]
        if (unit) {
          const mountPath = unit.subdirectory || `addons/${unit.name}`
          requests.push({ project_id: projectId, plugin_id: pluginId, version_id: version.version_id, unit_id: unit.unit_id, mount_path: mountPath })
        }
      }
    }
  }
  try {
    const result = await api.batchBindPlugins(requests)
    if (result.failed_count > 0) {
      toast.warning(t('common.batchDeleteComplete', { success: result.success_count, failed: result.failed_count }))
    } else {
      toast.success(t('plugins.bindDialog.bindAndApplySuccess', { count: result.success_count, name: '' }))
    }
    selectedLinkPluginIds.value = new Set()
    linkerHasPendingChanges.value = true
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLinkerBatchBinding.value = false
    showLinkerBatchBindDialog.value = false
  }
}

const batchUnbindPlugins = async () => {
  if (!selectedLinkId.value) {
    toast.warning(t('linker.selectProject'))
    return
  }
  const boundSelected = linkerBindings.value.filter(b => selectedLinkPluginIds.value.has(b.plugin_id))
  if (boundSelected.length === 0) {
    toast.warning(t('linker.noBindings'))
    return
  }
  showLinkerBatchUnbindDialog.value = true
}

const confirmBatchUnbind = async () => {
  if (!selectedLinkId.value) return
  isLinkerBatchUnbinding.value = true
  const pluginIds = linkerBindings.value
    .filter(b => selectedLinkPluginIds.value.has(b.plugin_id))
    .map(b => b.plugin_id)
  try {
    const result = await api.batchUnbindPlugins(selectedLinkId.value, pluginIds)
    if (result.failed_count > 0) {
      toast.warning(t('common.batchDeleteComplete', { success: result.success_count, failed: result.failed_count }))
    } else {
      toast.success(t('linker.pluginUnbound'))
    }
    selectedLinkPluginIds.value = new Set()
    linkerHasPendingChanges.value = true
    await loadLinkerBindings(selectedLinkId.value)
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLinkerBatchUnbinding.value = false
    showLinkerBatchUnbindDialog.value = false
  }
}

const batchApplyChanges = () => {
  if (selectedLinkProjectIds.value.size === 0) {
    toast.warning(t('linker.selectProject'))
    return
  }
  showLinkerBatchApplyDialog.value = true
}

const confirmBatchApply = async () => {
  isLinkerBatchApplying.value = true
  batchApplyResults.value = []
  const projectIds = Array.from(selectedLinkProjectIds.value)
  for (const projectId of projectIds) {
    try {
      const result = await api.applyChanges(projectId)
      const project = linkerProjects.value.find(p => p.project_id === projectId)
      batchApplyResults.value.push({ project_id: projectId, project_name: project?.name || projectId, ...result })
    } catch (error) {
      const project = linkerProjects.value.find(p => p.project_id === projectId)
      batchApplyResults.value.push({ project_id: projectId, project_name: project?.name || projectId, success: false, errors: [String(error)], created: [], removed: [] })
    }
  }
  isLinkerBatchApplying.value = false
  showLinkerBatchApplyDialog.value = false
  showLinkerBatchApplyResult.value = true
  linkerHasPendingChanges.value = false
  if (selectedLinkId.value) {
    await loadLinkerBindings(selectedLinkId.value)
  }
}

const selectedLinkProjectCount = computed(() => selectedLinkProjectIds.value.size)
const selectedLinkPluginCount = computed(() => selectedLinkPluginIds.value.size)

const boundPluginNames = computed(() => {
  return linkerBindings.value.map(b => {
    const plugin = plugins.value.find(p => p.plugin_id === b.plugin_id)
    return { ...b, plugin }
  })
})

const unboundPlugins = computed(() => {
  const boundIds = new Set(linkerBindings.value.map(b => b.plugin_id))
  return plugins.value.filter(p => !boundIds.has(p.plugin_id))
})

const filteredUnboundPlugins = computed(() => {
  const query = linkerSearchQuery.value.toLowerCase().trim()
  if (!query) return unboundPlugins.value
  return unboundPlugins.value.filter(p =>
    p.name.toLowerCase().includes(query) ||
    p.description.toLowerCase().includes(query) ||
    p.author.toLowerCase().includes(query)
  )
})

const toggleLinkPluginSelection = (pluginId: string, event: MouseEvent) => {
  if (event.ctrlKey || event.metaKey) {
    if (selectedLinkPluginIds.value.has(pluginId)) {
      selectedLinkPluginIds.value.delete(pluginId)
    } else {
      selectedLinkPluginIds.value.add(pluginId)
    }
  } else {
    if (selectedLinkPluginIds.value.has(pluginId)) {
      selectedLinkPluginIds.value.delete(pluginId)
    } else {
      selectedLinkPluginIds.value.add(pluginId)
    }
  }
  selectedLinkPluginIds.value = new Set(selectedLinkPluginIds.value)
}

const graphNodes = computed(() => {
  const nodes: { type: 'project' | 'plugin', id: string, name: string, y: number }[] = []
  const projs = linkerProjects.value
  const plgs = plugins.value
  projs.forEach((p, i) => {
    nodes.push({ type: 'project', id: p.project_id, name: p.name, y: i * 40 + 30 })
  })
  plgs.forEach((p, i) => {
    nodes.push({ type: 'plugin', id: p.plugin_id, name: p.name, y: i * 40 + 30 })
  })
  return nodes
})

const graphLinks = computed(() => {
  if (!selectedLinkId.value) return []
  return linkerBindings.value.map(b => ({
    projectId: b.project_id,
    pluginId: b.plugin_id,
    projectName: linkerProjects.value.find(p => p.project_id === b.project_id)?.name || '',
    pluginName: plugins.value.find(p => p.plugin_id === b.plugin_id)?.name || ''
  }))
})

useDialogEscape(showLinkerApplyDialog)
useDialogEscape(showLinkerBatchBindDialog)
useDialogEscape(showLinkerBatchUnbindDialog)
useDialogEscape(showLinkerBatchApplyDialog)
useDialogEscape(showLinkerVersionSelect)
useDialogEscape(showLinkerApplyResult)
useDialogEscape(showLinkerBatchApplyResult)

</script>

<template>
  <div class="space-y-6">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div class="flex items-center gap-4">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('plugins.title') }}</h1>
        <div class="flex bg-gray-100 dark:bg-surface-layer rounded-lg p-1">
          <button
            @click="activeTab = 'repository'"
            :class="['px-3 py-1.5 rounded-md text-sm font-medium transition-colors', activeTab === 'repository' ? 'bg-white dark:bg-surface-card text-primary-600 dark:text-primary-400 shadow-sm' : 'text-gray-600 dark:text-content-secondary hover:text-gray-900 dark:hover:text-content-primary']"
          >{{ t('plugins.tabRepository') }}</button>
          <button
            @click="activeTab = 'bindings'; loadLinkerData()"
            :class="['px-3 py-1.5 rounded-md text-sm font-medium transition-colors', activeTab === 'bindings' ? 'bg-white dark:bg-surface-card text-primary-600 dark:text-primary-400 shadow-sm' : 'text-gray-600 dark:text-content-secondary hover:text-gray-900 dark:hover:text-content-primary']"
          >{{ t('plugins.tabBindings') }}</button>
        </div>
      </div>
      <div v-if="activeTab === 'repository'" class="flex flex-wrap gap-2">
        <button
          @click="checkPluginUpdates"
          :disabled="isCheckingUpdates || isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ isCheckingUpdates ? t('plugins.checkingUpdates') : t('plugins.checkUpdates') }}
        </button>
        <div class="relative" ref="addMenuRef">
          <button
            @click="showAddMenu = !showAddMenu"
            :disabled="isLoading"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm flex items-center gap-1.5"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            {{ t('plugins.addPlugin') }}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div v-if="showAddMenu" class="absolute right-0 mt-2 w-56 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50 py-1">
            <button
              @click="importFromLocal(); showAddMenu = false"
              class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2.5"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              <div>
                <div class="font-medium">{{ t('plugins.fromDir') }}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.addMenu.fromDirDesc') }}</div>
              </div>
            </button>
            <button
              @click="importFromFile(); showAddMenu = false"
              class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2.5"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <div>
                <div class="font-medium">{{ t('plugins.fromFile') }}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.addMenu.fromFileDesc') }}</div>
              </div>
            </button>
            <button
              @click="showGitDialog = true; showAddMenu = false"
              class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2.5"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
              <div>
                <div class="font-medium">{{ t('plugins.fromGit') }}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.addMenu.fromGitDesc') }}</div>
              </div>
            </button>
            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
            <button
              @click="importFromProjects(); showAddMenu = false"
              class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2.5"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              <div>
                <div class="font-medium">{{ t('plugins.fromProjects') }}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.addMenu.fromProjectsDesc') }}</div>
              </div>
            </button>
            <button
              @click="openAssetLibrary(); showAddMenu = false"
              class="w-full text-left px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2.5"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <div>
                <div class="font-medium">{{ t('assetLibrary.title') }}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.addMenu.fromAssetLibDesc') }}</div>
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'bindings'" class="card">
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
            <option value="Both">{{ t('plugins.compat.both') }}</option>
          </select>
          <select
            v-model="filterSource"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">{{ t('plugins.allSource') }}</option>
            <option value="Local">{{ t('plugins.source.local') }}</option>
            <option value="Git">{{ t('plugins.source.git') }}</option>
            <option value="AssetLibrary">{{ t('plugins.source.assetlibrary') }}</option>
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
              {{ favoritePlugins }} {{ t('plugins.favorites') }}
            </span>
          </button>
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="filteredPlugins.length === 0 && plugins.length === 0" class="text-center py-12 max-w-md mx-auto">
      <div class="w-16 h-16 mx-auto mb-4 bg-primary-100 dark:bg-primary-900/30 rounded-2xl flex items-center justify-center">
        <svg class="w-10 h-10 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
        </svg>
      </div>
      <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('plugins.onboarding.title') }}</h3>
      <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">{{ t('plugins.onboarding.desc') }}</p>
      <div class="mt-6 space-y-3">
        <button
          @click="importFromLocal"
          :disabled="isLoading"
          class="w-full flex items-center gap-3 p-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-left"
        >
          <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center flex-shrink-0">
            <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </div>
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('plugins.onboarding.fromDir') }}</div>
            <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.onboarding.fromDirDesc') }}</div>
          </div>
        </button>
        <button
          @click="showGitDialog = true"
          :disabled="isLoading"
          class="w-full flex items-center gap-3 p-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-left"
        >
          <div class="w-10 h-10 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center flex-shrink-0">
            <svg class="w-5 h-5 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
          </div>
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('plugins.onboarding.fromGit') }}</div>
            <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.onboarding.fromGitDesc') }}</div>
          </div>
        </button>
        <button
          @click="openAssetLibrary"
          :disabled="isLoading"
          class="w-full flex items-center gap-3 p-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-left"
        >
          <div class="w-10 h-10 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center flex-shrink-0">
            <svg class="w-5 h-5 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('plugins.onboarding.fromAssetLib') }}</div>
            <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.onboarding.fromAssetLibDesc') }}</div>
          </div>
        </button>
        <button
          @click="importFromProjects"
          :disabled="isLoading"
          class="w-full flex items-center gap-3 p-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-left"
        >
          <div class="w-10 h-10 bg-amber-100 dark:bg-amber-900/30 rounded-lg flex items-center justify-center flex-shrink-0">
            <svg class="w-5 h-5 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </div>
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('plugins.onboarding.fromProjects') }}</div>
            <div class="text-xs text-gray-500 dark:text-gray-400">{{ t('plugins.onboarding.fromProjectsDesc') }}</div>
          </div>
        </button>
      </div>
    </div>

    <div v-else-if="filteredPlugins.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('plugins.empty') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{{ t('plugins.emptyDesc') }}</p>
    </div>

    <div v-else class="space-y-4">
      <div v-if="isBatchMode && selectedPluginCount > 0" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-lg p-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span class="text-sm font-medium text-primary-700 dark:text-primary-300">{{ t('plugins.selectedCount', { count: selectedPluginCount }) }}</span>
          <button
            @click="selectAllPlugins"
            class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
          >
            {{ t('plugins.batchActions.selectAll') }}
          </button>
          <button
            @click="clearPluginSelection"
            class="text-xs text-gray-500 dark:text-gray-400 hover:underline"
          >
            {{ t('plugins.batchActions.deselectAll') }}
          </button>
        </div>
        <div class="flex gap-2">
          <button
            @click="batchRemovePlugins"
            class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-lg hover:bg-red-700 transition-colors"
          >
            {{ t('plugins.batchActions.batchDelete', { count: selectedPluginCount }) }}
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
              {{ plugin.compatibility === 'Godot4' ? 'Godot 4' : plugin.compatibility === 'Godot3' ? 'Godot 3' : plugin.compatibility === 'Both' ? t('plugins.compat.both') : t('plugins.compat.unknown') }}
            </span>
            <span class="badge badge-neutral">
              {{ plugin.source.source_type === 'Local' ? t('plugins.source.local') : plugin.source.source_type === 'Git' ? t('plugins.source.git') : t('plugins.source.assetlibrary') }}
            </span>
          </div>
          <div class="mt-3 pt-3 border-t border-gray-100 dark:border-gray-700 flex gap-2">
            <button
              @click.stop="openBindDialog(plugin)"
              class="flex-1 px-3 py-1.5 bg-primary-600 text-white text-xs rounded-lg hover:bg-primary-700 transition-colors flex items-center justify-center gap-1.5"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
              {{ t('plugins.bindToProject') }}
            </button>
            <button
              @click.stop="goToBindings(plugin)"
              class="px-2 py-1.5 border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-content-secondary text-xs rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
              :title="t('plugins.goToBindings')"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showGitDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showGitDialog = false; gitUrl = ''">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('plugins.importFromGit') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">
          {{ t('plugins.gitImport.desc') }}
        </p>
        <input
          v-model="gitUrl"
          type="text"
          :placeholder="t('plugins.gitImport.placeholder')"
          class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
        />
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showGitDialog = false; gitUrl = ''"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="importFromGit"
            :disabled="isLoading || !gitUrl"
            class="btn-primary disabled:opacity-50"
          >
            {{ t('plugins.importFromProject.startImport') }}
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
          <span>{{ selectedPlugin.compatibility === 'Godot4' ? 'Godot 4' : selectedPlugin.compatibility === 'Godot3' ? 'Godot 3' : selectedPlugin.compatibility === 'Both' ? t('plugins.compat.both') : t('plugins.compat.unknown') }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ selectedPlugin.source.source_type === 'Local' ? t('plugins.source.local') : selectedPlugin.source.source_type === 'Git' ? t('plugins.source.git') : t('plugins.source.assetlibrary') }}</span>
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
              {{ t('plugins.pluginDetail.versionList', { count: selectedPlugin.versions.length }) }}
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
                    {{ t('plugins.pluginDetail.unitCount', { count: version.units.length }) }}
                  </span>
                </div>
                <button
                  v-if="selectedPlugin.versions.length > 1"
                  @click="removePluginVersion(selectedPlugin.plugin_id, version.version_id)"
                  class="text-xs text-red-500 hover:text-red-700"
                >
                  {{ t('common.delete') }}
                </button>
              </div>
            </div>
          </div>

          <div v-if="pluginBindings.length > 0">
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">
              {{ t('plugins.pluginDetail.bindings', { count: pluginBindings.length }) }}
            </h4>
            <div class="space-y-1 bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              <div v-for="binding in pluginBindings" :key="binding.project_id + binding.mount_path"
                class="flex items-center justify-between py-1">
                <div class="flex items-center gap-2 min-w-0">
                  <span v-if="binding.is_healthy === false" class="inline-flex items-center gap-1 text-xs text-red-500 flex-shrink-0">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                    </svg>
                    {{ t('plugins.bindDialog.broken') }}
                  </span>
                  <span v-else-if="binding.is_healthy === true" class="inline-flex items-center gap-1 text-xs text-green-500 flex-shrink-0">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                  </span>
                  <span class="text-xs text-gray-900 dark:text-content-primary font-medium truncate">{{ bindingProjects.get(binding.project_id) || binding.project_id }}</span>
                  <span class="font-mono text-xs text-gray-500 dark:text-content-secondary flex-shrink-0">{{ binding.mount_path }}</span>
                </div>
                <button
                  v-if="binding.is_healthy === false"
                  @click="repairBinding(binding.project_id, binding.plugin_id)"
                  class="text-xs text-primary-600 dark:text-primary-400 hover:underline flex-shrink-0 ml-2"
                >
                  {{ t('plugins.bindDialog.repair') }}
                </button>
              </div>
            </div>
          </div>

          <div v-if="pluginDependencies.length > 0">
            <div class="flex items-center justify-between mb-2">
              <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary">{{ t('plugins.pluginDetail.dependencies') }}</h4>
              <button
                v-if="missingDepPluginIds.length > 0"
                @click="installMissingDeps"
                :disabled="isInstallingDeps"
                class="text-xs text-primary-600 dark:text-primary-400 hover:underline disabled:opacity-50"
              >
                {{ isInstallingDeps ? t('plugins.depDialog.installing') : t('plugins.depDialog.installMissing', { count: missingDepPluginIds.length }) }}
              </button>
            </div>
            <div class="space-y-2 bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              <div v-for="dep in pluginDependencies" :key="dep.plugin_id" class="flex items-center justify-between text-sm">
                <div class="text-gray-600 dark:text-content-secondary">
                  <span class="font-medium">{{ dep.plugin_id }}</span>
                  <span v-if="dep.version_constraint"> ({{ dep.version_constraint }})</span>
                  <span v-if="dep.is_optional" class="ml-2 text-xs text-gray-500 dark:text-content-secondary">({{ t('plugins.pluginDetail.optional') }})</span>
                </div>
                <span v-if="!dep.is_optional && !installedPluginIds.has(dep.plugin_id)" class="text-xs text-red-500">{{ t('plugins.depDialog.missing') }}</span>
                <span v-else-if="installedPluginIds.has(dep.plugin_id)" class="text-xs text-green-500">✓</span>
              </div>
            </div>
          </div>

          <div>
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('plugins.pluginDetail.source') }}</h4>
            <p class="text-sm text-gray-600 dark:text-content-secondary bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
              {{ t(`plugins.pluginDetail.sourceTypes.${selectedPlugin.source.source_type}`) }}
              <span v-if="selectedPlugin.source.url" class="block text-xs mt-1 break-all font-mono">{{ selectedPlugin.source.url }}</span>
            </p>
          </div>

          <div v-if="pluginStorageStats" class="grid grid-cols-3 gap-3">
            <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 text-center">
              <div class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.version_count }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">{{ t('plugins.pluginDetail.sections.version') }}</div>
            </div>
            <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 text-center">
              <div class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.binding_count }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">{{ t('plugins.pluginDetail.sections.mount') }}</div>
            </div>
            <div class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 text-center">
              <div class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.total_size_display }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">{{ t('plugins.pluginDetail.sections.storage') }}</div>
            </div>
          </div>
        </div>

        <div class="flex justify-end mt-4 pt-3 border-t border-gray-200 dark:border-gray-700">
          <button
            @click="closePluginDetail"
            class="btn-secondary"
          >
            {{ t('common.close') }}
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
            <option value="any">{{ t('assetLibrary.godotVersionAny') }}</option>
            <option value="4.0">{{ t('assetLibrary.godot4x') }}</option>
            <option value="3.0">{{ t('assetLibrary.godot3x') }}</option>
          </select>
          <select v-model="assetFilterSupport" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
            <option value="">{{ t('assetLibrary.supportAll') }}</option>
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
                v-if="!importedAssetIds.has(asset.asset_id)"
                @click="importAsset(asset.asset_id, asset.title)"
                :disabled="pluginStore.isImporting === asset.asset_id"
                class="btn-primary disabled:opacity-50 text-xs px-3 py-1.5 flex-shrink-0"
              >
                {{ pluginStore.isImporting === asset.asset_id ? t('assetLibrary.importing') : t('assetLibrary.import') }}
              </button>
              <span v-else class="text-xs px-3 py-1.5 text-green-600 dark:text-green-400 flex-shrink-0 font-medium">✓ {{ t('assetLibrary.alreadyImported') }}</span>
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
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('plugins.updateCheck.title') }}</h3>
          <button @click="showUpdatesDialog = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="space-y-3 max-h-80 overflow-y-auto">
          <div v-if="pluginUpdates.length === 0" class="text-center py-8 text-gray-500 dark:text-gray-400">
            {{ t('plugins.updateCheck.noPlugins') }}
          </div>
          <div v-for="update in pluginUpdates" :key="update.plugin_id" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <span class="font-medium text-gray-900 dark:text-gray-100">{{ update.plugin_id }}</span>
                <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                  {{ t('plugins.updateCheck.versionInfo', { current: update.current_version, latest: update.latest_version }) }}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button
                  v-if="update.update_available"
                  @click="updateGitPlugin(update.plugin_id)"
                  :disabled="isBatchUpdating"
                  class="px-3 py-1 bg-primary-600 text-white text-xs rounded-lg hover:bg-primary-700 disabled:opacity-50"
                >
                  {{ t('plugins.updateCheck.update') }}
                </button>
                <span v-if="update.update_available" class="px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  {{ t('plugins.updateCheck.hasUpdate') }}
                </span>
                <span v-else class="px-2 py-1 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-600 dark:text-gray-400">
                  {{ t('plugins.updateCheck.upToDate') }}
                </span>
              </div>
            </div>
            <div v-if="update.release_notes" class="mt-2 pt-2 border-t border-gray-200 dark:border-gray-600">
              <button
                @click="expandedReleaseNotes.has(update.plugin_id) ? expandedReleaseNotes.delete(update.plugin_id) : expandedReleaseNotes.add(update.plugin_id)"
                class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
              >
                {{ expandedReleaseNotes.has(update.plugin_id) ? t('plugins.updateCheck.hideNotes') : t('plugins.updateCheck.showNotes') }}
              </button>
              <div v-if="expandedReleaseNotes.has(update.plugin_id)" class="mt-2 text-xs text-gray-600 dark:text-gray-300 whitespace-pre-wrap max-h-32 overflow-y-auto bg-white dark:bg-gray-800 rounded p-2">
                {{ update.release_notes }}
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-between mt-4">
          <button
            v-if="updatablePluginIds.length > 0"
            @click="batchUpdatePlugins"
            :disabled="isBatchUpdating"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 text-sm"
          >
            {{ isBatchUpdating ? t('plugins.updateCheck.updating') : t('plugins.updateCheck.updateAll', { count: updatablePluginIds.length }) }}
          </button>
          <div v-else></div>
          <button
            @click="showUpdatesDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showScanPreviewDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showScanPreviewDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.importFromProject.scanTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">{{ t('plugins.importFromProject.scanDesc', { count: scannedPlugins.length }) }}</p>
        <div class="flex-1 overflow-y-auto space-y-2 mb-4">
          <div v-for="(plugin, idx) in scannedPlugins" :key="idx" class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 flex items-center gap-3">
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ plugin.plugin_name }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary truncate">{{ plugin.project_name }} · {{ plugin.path }}</div>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showScanPreviewDialog = false" class="btn-secondary">{{ t('common.cancel') }}</button>
          <button @click="startImportFromPreview" class="btn-primary">{{ t('plugins.importFromProject.continueImport') }}</button>
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

    <div v-if="activeTab === 'repository' && totalStorageStats" class="card">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4 text-sm text-gray-600 dark:text-content-secondary">
          <span>{{ t('plugins.storageStats.plugins', { count: totalStorageStats.total_plugins }) }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ t('plugins.storageStats.versions', { count: totalStorageStats.total_versions }) }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ t('plugins.storageStats.bindings', { count: totalStorageStats.total_bindings }) }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ t('plugins.storageStats.size', { size: totalStorageStats.total_size_display }) }}</span>
          <span v-if="totalStorageStats.orphaned_size_bytes > 0" class="text-orange-500">
            | {{ t('plugins.storageStats.orphaned', { size: totalStorageStats.orphaned_size_display }) }}
          </span>
          <span v-if="totalStorageStats.duplicate_hash_count > 0" class="text-yellow-500">
            | {{ t('plugins.storageStats.duplicates', { count: totalStorageStats.duplicate_hash_count }) }}
          </span>
        </div>
        <button
          v-if="totalStorageStats.orphaned_size_bytes > 0"
          @click="cleanupOrphaned"
          class="px-3 py-1 text-xs border border-orange-300 dark:border-orange-700 text-orange-600 dark:text-orange-400 rounded-lg hover:bg-orange-50 dark:hover:bg-orange-900/20"
        >
          {{ t('plugins.storageStats.cleanup') }}
        </button>
      </div>
    </div>

    <div v-if="activeTab === 'bindings'" class="space-y-4">
      <div class="flex flex-wrap gap-2 items-center">
        <button
          @click="showGraphView = !showGraphView"
          class="px-3 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-layer"
        >
          {{ showGraphView ? t('linker.listView') : t('linker.graphView') }}
        </button>
        <button
          v-if="selectedLinkProjectCount > 1"
          @click="batchApplyChanges"
          class="px-3 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-layer"
        >
          {{ t('linker.batchApplyTitle') }}
        </button>
        <button
          v-if="selectedLinkId"
          @click="confirmLinkerApply"
          :disabled="isLinkerApplying"
          class="relative px-3 py-1.5 text-sm rounded-lg hover:bg-primary-700 disabled:opacity-50"
          :class="linkerHasPendingChanges ? 'bg-orange-500 text-white animate-pulse' : 'bg-primary-600 text-white'"
        >
          {{ isLinkerApplying ? t('linker.applying') : t('linker.apply') }}
          <span v-if="linkerHasPendingChanges" class="absolute -top-1 -right-1 w-3 h-3 bg-red-500 rounded-full border-2 border-white dark:border-gray-900"></span>
        </button>
      </div>

      <div v-if="linkerHasPendingChanges" class="bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-lg p-2.5 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 text-orange-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
          <span class="text-sm text-orange-700 dark:text-orange-300">{{ t('linker.pendingChanges') }}</span>
        </div>
        <button
          v-if="selectedLinkId"
          @click="confirmLinkerApply"
          :disabled="isLinkerApplying"
          class="px-3 py-1 bg-orange-500 text-white text-xs rounded-lg hover:bg-orange-600 disabled:opacity-50"
        >
          {{ isLinkerApplying ? t('linker.applying') : t('linker.applyNow') }}
        </button>
      </div>

      <div v-if="!showGraphView" class="grid grid-cols-12 gap-4">
        <div class="col-span-3 card p-0">
          <div class="p-3 border-b border-gray-200 dark:border-surface-border">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('linker.projectList') }}</h3>
          </div>
          <div class="max-h-[calc(100vh-280px)] overflow-y-auto">
            <div v-if="linkerProjects.length === 0" class="p-4 text-center">
              <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('linker.emptyProject') }}</h3>
              <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">{{ t('linker.emptyProjectDesc') }}</p>
            </div>
            <div
              v-for="project in linkerProjects"
              :key="project.project_id"
              @click="selectLinkProject(project, $event)"
              :class="['px-3 py-2.5 cursor-pointer border-b border-gray-100 dark:border-surface-border last:border-0 transition-colors', selectedLinkProjectIds.has(project.project_id) ? 'bg-primary-50 dark:bg-primary-900/20' : 'hover:bg-gray-50 dark:hover:bg-surface-layer']"
            >
              <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ project.name }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary truncate">{{ project.godot_version }}</div>
            </div>
          </div>
        </div>

        <div class="col-span-5 card p-0">
          <div class="p-3 border-b border-gray-200 dark:border-surface-border">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('linker.availablePlugins') }}</h3>
              <button
                v-if="selectedLinkProjectCount > 0 && selectedLinkPluginCount > 0"
                @click="batchBindPlugins"
                class="px-2 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700"
              >
                {{ t('linker.batchBind') }}
              </button>
            </div>
            <input
              v-model="linkerSearchQuery"
              type="text"
              :placeholder="t('linker.searchPlugins')"
              class="w-full px-2.5 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary text-xs"
            />
          </div>
          <div class="max-h-[calc(100vh-280px)] overflow-y-auto">
            <div v-if="selectedLinkProjectCount === 0" class="p-4 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ t('linker.selectProject') }}
            </div>
            <div v-else-if="filteredUnboundPlugins.length === 0" class="p-4 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ linkerSearchQuery ? t('linker.noSearchResults') : t('linker.allPluginsBound') }}
            </div>
            <div
              v-for="plugin in filteredUnboundPlugins"
              :key="plugin.plugin_id"
              @click="toggleLinkPluginSelection(plugin.plugin_id, $event)"
              :class="['px-3 py-2.5 cursor-pointer border-b border-gray-100 dark:border-surface-border last:border-0 transition-colors', selectedLinkPluginIds.has(plugin.plugin_id) ? 'bg-primary-50 dark:bg-primary-900/20' : 'hover:bg-gray-50 dark:hover:bg-surface-layer']"
            >
              <div class="flex items-center justify-between">
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ plugin.name }}</div>
                  <div class="text-xs text-gray-500 dark:text-content-secondary">v{{ plugin.versions[0]?.version || '1.0.0' }} · {{ plugin.author || t('linker.unknown') }}</div>
                </div>
                <button @click.stop="bindPluginToProject(plugin)" class="px-2 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 ml-2 flex-shrink-0">
                  {{ t('linker.bind') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="col-span-4 card p-0">
          <div class="p-3 border-b border-gray-200 dark:border-surface-border flex items-center justify-between">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('linker.boundPlugins') }}</h3>
            <button
              v-if="boundPluginNames.some(b => selectedLinkPluginIds.has(b.plugin_id))"
              @click="batchUnbindPlugins"
              class="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700"
            >
              {{ t('linker.batchUnbind') }}
            </button>
          </div>
          <div class="max-h-[calc(100vh-280px)] overflow-y-auto">
            <div v-if="!selectedLinkId" class="p-4 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ t('linker.selectProject') }}
            </div>
            <div v-else-if="boundPluginNames.length === 0" class="p-4 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ t('linker.noBindings') }}
            </div>
            <div
              v-for="item in boundPluginNames"
              :key="`${item.project_id}-${item.plugin_id}`"
              :class="['px-3 py-2.5 border-b border-gray-100 dark:border-surface-border last:border-0', item.is_healthy === false ? 'bg-red-50 dark:bg-red-900/10' : '']"
            >
              <div class="flex items-center justify-between">
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1.5">
                    {{ item.plugin?.name || t('linker.unknownPlugin') }}
                    <span v-if="item.is_healthy === false" class="w-2 h-2 rounded-full bg-red-500 flex-shrink-0" :title="t('plugins.bindDialog.unhealthy')"></span>
                  </div>
                  <div class="text-xs text-gray-500 dark:text-content-secondary">{{ item.mount_path }}</div>
                </div>
                <button @click="unbindPluginFromProject(item)" class="px-2 py-1 text-red-600 dark:text-red-400 text-xs hover:bg-red-50 dark:hover:bg-red-900/20 rounded ml-2 flex-shrink-0">
                  {{ t('linker.unbind') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="card p-4">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('linker.bindingGraph') }}</h3>
        <svg width="100%" height="400" viewBox="0 0 640 400" class="border border-gray-200 dark:border-surface-border rounded-lg">
          <text x="80" y="20" text-anchor="middle" class="fill-gray-500 dark:fill-gray-400" font-size="11">{{ t('linker.projects') }}</text>
          <text x="560" y="20" text-anchor="middle" class="fill-gray-500 dark:fill-gray-400" font-size="11">{{ t('linker.plugins') }}</text>
          <g v-for="node in graphNodes.filter(n => n.type === 'project')" :key="'p-' + node.id">
            <rect :x="10" :y="node.y - 12" width="140" height="24" rx="4" class="fill-blue-100 dark:fill-blue-900/30 stroke-blue-300 dark:stroke-blue-700" stroke-width="1"/>
            <text :x="80" :y="node.y + 4" text-anchor="middle" class="fill-gray-700 dark:fill-gray-300" font-size="10">{{ node.name.substring(0, 12) }}</text>
          </g>
          <g v-for="node in graphNodes.filter(n => n.type === 'plugin')" :key="'pl-' + node.id">
            <rect :x="490" :y="node.y - 12" width="140" height="24" rx="4" class="fill-green-100 dark:fill-green-900/30 stroke-green-300 dark:stroke-green-700" stroke-width="1"/>
            <text :x="560" :y="node.y + 4" text-anchor="middle" class="fill-gray-700 dark:fill-gray-300" font-size="10">{{ node.name.substring(0, 12) }}</text>
          </g>
          <line v-for="link in graphLinks" :key="link.projectId + '-' + link.pluginId" :x1="150" :y1="graphNodes.find(n => n.id === link.projectId)?.y" :x2="490" :y2="graphNodes.find(n => n.id === link.pluginId)?.y" class="stroke-gray-400 dark:stroke-gray-500" stroke-width="1" stroke-dasharray="4"/>
        </svg>
      </div>
    </div>

    <div v-if="showDeletePluginConfirm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showDeletePluginConfirm = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('plugins.deleteConfirm.single') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-3">
          {{ t('plugins.deleteConfirm.singleDesc') }}
        </p>
        <div v-if="deletePluginBindings.length > 0" class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm font-medium text-red-700 dark:text-red-400 mb-2">
            {{ t('plugins.deleteConfirm.bindingWarning', { count: new Set(deletePluginBindings.map(b => b.project_id)).size, name: deletePluginName }) }}
          </p>
          <div class="space-y-2 max-h-40 overflow-y-auto">
            <div v-for="projectId in [...new Set(deletePluginBindings.map(b => b.project_id))]" :key="projectId" class="text-xs"><div class="font-medium text-red-700 dark:text-red-400">{{ deletePluginProjects.get(projectId) || projectId }}</div><div v-for="binding in deletePluginBindings.filter(b => b.project_id === projectId)" :key="binding.mount_path" class="text-red-600 dark:text-red-400 pl-3">{{ binding.mount_path }}</div></div>
          </div>
          <p class="text-xs text-red-500 dark:text-red-400 mt-2">
            {{ t('plugins.deleteConfirm.bindingWarningDesc') }}
          </p>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showDeletePluginConfirm = false" class="btn-secondary">{{ t('common.cancel') }}</button>
          <button @click="onRemovePluginConfirm(); showDeletePluginConfirm = false" class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 text-sm">{{ t('plugins.deleteConfirm.singleConfirm') }}</button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showBatchDeleteConfirm"
      :title="t('plugins.deleteConfirm.batch')"
      :description="t('plugins.deleteConfirm.batchDesc', { count: selectedPluginCount })"
      :confirm-text="t('plugins.deleteConfirm.batchConfirm')"
      @confirm="onBatchDeleteConfirm"
    />

    <div v-if="showDuplicateConfirm && duplicateCheckResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showDuplicateConfirm = false; duplicateCheckResult = null; pendingImportAction = null">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('plugins.duplicate.title') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">
          {{ t('plugins.duplicate.desc', { name: duplicateCheckResult.duplicate_plugin_name || duplicateCheckResult.duplicate_plugin_id || '' }) }}
        </p>
        <div class="flex justify-end gap-3">
          <button @click="showDuplicateConfirm = false; duplicateCheckResult = null; pendingImportAction = null" class="btn-secondary">{{ t('plugins.duplicate.cancel') }}</button>
          <button
            @click="showDuplicateConfirm = false; duplicateCheckResult = null; if (pendingImportAction) { pendingImportAction(); pendingImportAction = null; }"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm"
          >
            {{ t('plugins.duplicate.importAnyway') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showBindDialog && bindTargetPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBindDialog = false; bindTargetPlugin = null">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
            {{ t('plugins.bindDialog.title', { name: bindTargetPlugin.name }) }}
          </h3>
          <button @click="showBindDialog = false; bindTargetPlugin = null" class="text-gray-500 dark:text-content-secondary hover:text-gray-700 dark:hover:text-content-primary">
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="flex-1 overflow-y-auto space-y-4">
          <div>
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('plugins.bindDialog.selectProjects') }}</h4>
            <div v-if="bindProjects.length === 0" class="text-sm text-gray-500 dark:text-content-secondary py-4 text-center">
              {{ t('plugins.bindDialog.noProjects') }}
            </div>
            <div v-else class="space-y-1 max-h-48 overflow-y-auto">
              <label
                v-for="project in bindProjects"
                :key="project.project_id"
                class="flex items-center gap-2 p-2 rounded-lg cursor-pointer transition-colors"
                :class="bindSelectedProjectIds.has(project.project_id) ? 'bg-primary-50 dark:bg-primary-900/20' : 'hover:bg-gray-50 dark:hover:bg-surface-layer'"
              >
                <input
                  type="checkbox"
                  :checked="bindSelectedProjectIds.has(project.project_id)"
                  @change="(() => { const s = new Set(bindSelectedProjectIds); s.has(project.project_id) ? s.delete(project.project_id) : s.add(project.project_id); bindSelectedProjectIds = s; })"
                  class="w-4 h-4 text-primary-600 rounded flex-shrink-0 cursor-pointer"
                />
                <div class="min-w-0 flex-1">
                  <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ project.name }}</span>
                  <span class="text-xs text-gray-500 dark:text-content-secondary ml-2">{{ project.godot_version }}</span>
                </div>
              </label>
            </div>
          </div>

          <div v-if="bindTargetPlugin.versions.length > 1">
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('plugins.bindDialog.selectVersion') }}</h4>
            <select
              v-model="bindSelectedVersionIdx"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
            >
              <option v-for="(ver, idx) in bindTargetPlugin.versions" :key="ver.version_id" :value="idx">
                v{{ ver.version }} ({{ new Date(ver.created_at).toLocaleDateString() }})
              </option>
            </select>
          </div>

          <div v-if="bindTargetPlugin.versions[bindSelectedVersionIdx]?.units.length > 1">
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('plugins.bindDialog.selectUnit') }}</h4>
            <select
              v-model="bindSelectedUnitIdx"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
            >
              <option v-for="(unit, idx) in bindTargetPlugin.versions[bindSelectedVersionIdx]?.units" :key="unit.unit_id" :value="idx">
                {{ unit.name }}{{ unit.subdirectory ? ` (${unit.subdirectory})` : '' }}
              </option>
            </select>
          </div>

          <div class="text-xs text-gray-500 dark:text-content-secondary">
            {{ t('plugins.bindDialog.mountPath') }}: {{ bindTargetPlugin.versions[bindSelectedVersionIdx]?.units[bindSelectedUnitIdx]?.subdirectory || `addons/${bindTargetPlugin.versions[bindSelectedVersionIdx]?.units[bindSelectedUnitIdx]?.name || '?'}` }}
          </div>
        </div>

        <div class="flex justify-end gap-3 mt-4 pt-3 border-t border-gray-200 dark:border-gray-700">
          <button
            @click="showBindDialog = false; bindTargetPlugin = null"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="confirmBind(false)"
            :disabled="isBinding || bindSelectedProjectIds.size === 0"
            class="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 text-sm disabled:opacity-50"
          >
            {{ t('plugins.bindDialog.confirmBind', { count: bindSelectedProjectIds.size }) }}
          </button>
          <button
            @click="confirmBind(true)"
            :disabled="isBinding || bindSelectedProjectIds.size === 0"
            class="btn-primary disabled:opacity-50"
          >
            {{ isBinding ? t('plugins.bindDialog.binding') : t('plugins.bindDialog.bindAndApply', { count: bindSelectedProjectIds.size }) }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showLinkerVersionSelect && versionSelectPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerVersionSelect = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('linker.versionSelectTitle', { name: versionSelectPlugin.name }) }}</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('linker.selectVersion') }}</label>
            <select v-model="selectedVersionIdx" class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary text-sm">
              <option v-for="(v, i) in versionSelectPlugin.versions" :key="v.version_id" :value="i">{{ v.version }}</option>
            </select>
          </div>
          <div v-if="versionSelectPlugin.versions[selectedVersionIdx]?.units.length > 1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('linker.selectUnit') }}</label>
            <select v-model="selectedUnitIdx" class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-card text-gray-900 dark:text-content-primary text-sm">
              <option v-for="(u, i) in versionSelectPlugin.versions[selectedVersionIdx]?.units" :key="u.unit_id" :value="i">{{ u.name }}</option>
            </select>
          </div>
          <p class="text-xs text-gray-500 dark:text-content-secondary">
            {{ t('linker.mountPath') }}: {{ versionSelectPlugin.versions[selectedVersionIdx]?.units[selectedUnitIdx]?.subdirectory || `addons/${versionSelectPlugin.versions[selectedVersionIdx]?.units[selectedUnitIdx]?.name || '?'}` }}
          </p>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showLinkerVersionSelect = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmVersionSelect" class="btn-primary">{{ t('linker.confirmBind') }}</button>
        </div>
      </div>
    </div>

    <div v-if="showLinkerApplyResult && linkerApplyResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerApplyResult = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ linkerApplyResult.errors.length === 0 ? t('linker.applySuccess') : t('linker.applyWithErrors') }}</h3>
        <div v-if="linkerApplyResult.created.length > 0" class="mb-3">
          <p class="text-sm font-medium text-green-600 dark:text-green-400 mb-1">{{ t('linker.created') }}:</p>
          <ul class="text-xs text-gray-600 dark:text-gray-400 list-disc list-inside">
            <li v-for="item in linkerApplyResult.created" :key="item">{{ item }}</li>
          </ul>
        </div>
        <div v-if="linkerApplyResult.removed.length > 0" class="mb-3">
          <p class="text-sm font-medium text-yellow-600 dark:text-yellow-400 mb-1">{{ t('linker.removed') }}:</p>
          <ul class="text-xs text-gray-600 dark:text-gray-400 list-disc list-inside">
            <li v-for="item in linkerApplyResult.removed" :key="item">{{ item }}</li>
          </ul>
        </div>
        <div v-if="linkerApplyResult.errors.length > 0" class="mb-3">
          <p class="text-sm font-medium text-red-600 dark:text-red-400 mb-1">{{ t('linker.errors') }}:</p>
          <ul class="text-xs text-gray-600 dark:text-gray-400 list-disc list-inside">
            <li v-for="item in linkerApplyResult.errors" :key="item">{{ item }}</li>
          </ul>
        </div>
        <div class="flex justify-end">
          <button @click="showLinkerApplyResult = false" class="btn-primary">{{ t('linker.close') }}</button>
        </div>
      </div>
    </div>

    <div v-if="showLinkerBatchBindDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchBindDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('linker.batchBindTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">
          {{ t('linker.batchBindDesc', { projectCount: selectedLinkProjectCount, pluginCount: selectedLinkPluginCount }) }}
        </p>
        <div class="mb-4">
          <p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">{{ t('linker.targetProjects') }}：</p>
          <div class="flex flex-wrap gap-1">
            <span v-for="id in selectedLinkProjectIds" :key="id" class="text-xs bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 px-2 py-0.5 rounded">
              {{ linkerProjects.find(p => p.project_id === id)?.name || id }}
            </span>
          </div>
        </div>
        <div class="mb-4">
          <p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">{{ t('linker.bindPlugins') }}：</p>
          <div class="flex flex-wrap gap-1">
            <span v-for="id in selectedLinkPluginIds" :key="id" class="text-xs bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 px-2 py-0.5 rounded">
              {{ plugins.find(p => p.plugin_id === id)?.name || t('linker.unknownPlugin') }}
            </span>
          </div>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showLinkerBatchBindDialog = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmBatchBind" :disabled="isLinkerBatchBinding" class="btn-primary disabled:opacity-50">
            {{ isLinkerBatchBinding ? t('linker.batchBinding') : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showLinkerBatchUnbindDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchUnbindDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('linker.batchUnbindTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">
          {{ t('linker.batchUnbindDesc', { projectName: linkerProjects.find(p => p.project_id === selectedLinkId)?.name, count: linkerBindings.filter(b => selectedLinkPluginIds.has(b.plugin_id)).length }) }}
        </p>
        <div class="mb-4">
          <div class="flex flex-wrap gap-1">
            <span v-for="id in selectedLinkPluginIds" :key="id" class="text-xs bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 px-2 py-0.5 rounded">
              {{ plugins.find(p => p.plugin_id === id)?.name || t('linker.unknownPlugin') }}
            </span>
          </div>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showLinkerBatchUnbindDialog = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmBatchUnbind" :disabled="isLinkerBatchUnbinding" class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 text-sm disabled:opacity-50">
            {{ isLinkerBatchUnbinding ? t('linker.batchUnbinding') : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showLinkerBatchApplyDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchApplyDialog = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('linker.batchApplyTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-4">{{ t('linker.batchApplyDesc') }}</p>
        <div class="space-y-2 mb-4 max-h-48 overflow-y-auto">
          <div v-for="id in selectedLinkProjectIds" :key="id" class="flex items-center justify-between text-sm">
            <span class="text-gray-900 dark:text-content-primary">{{ linkerProjects.find(p => p.project_id === id)?.name || id }}</span>
            <span class="text-xs text-gray-500 dark:text-content-secondary">{{ t('linker.bindingCountUnit', { count: linkerBindings.filter(b => b.project_id === id).length }) }}</span>
          </div>
        </div>
        <div class="flex justify-end gap-3">
          <button @click="showLinkerBatchApplyDialog = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmBatchApply" :disabled="isLinkerBatchApplying" class="btn-primary disabled:opacity-50">
            {{ isLinkerBatchApplying ? t('linker.batchApplying') : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showLinkerBatchApplyResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchApplyResult = false">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('linker.batchApplyResultTitle') }}</h3>
        <div class="space-y-3 max-h-64 overflow-y-auto">
          <div v-for="result in batchApplyResults" :key="result.project_id" class="p-3 rounded-lg border" :class="result.success ? 'border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/10' : 'border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/10'">
            <div class="flex items-center justify-between mb-1">
              <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ result.project_name }}</span>
              <span class="text-xs" :class="result.success ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">{{ result.success ? t('linker.success') : t('linker.failed') }}</span>
            </div>
            <div v-if="result.created?.length > 0" class="text-xs text-green-600 dark:text-green-400">{{ t('linker.createdItem', { count: result.created.length }) }}</div>
            <div v-if="result.removed?.length > 0" class="text-xs text-yellow-600 dark:text-yellow-400">{{ t('linker.removedItem', { count: result.removed.length }) }}</div>
            <div v-if="result.errors?.length > 0" class="text-xs text-red-600 dark:text-red-400">{{ t('linker.errorList', { errors: result.errors.join(', ') }) }}</div>
          </div>
        </div>
        <div class="flex justify-end mt-4">
          <button @click="showLinkerBatchApplyResult = false" class="btn-primary">{{ t('linker.close') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>