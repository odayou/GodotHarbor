<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api'
import type { Plugin, Project, PluginDependency, PluginStorageStats, ProjectBinding, TotalStorageStats, DuplicateCheckResult, ScannedPlugin, FeaturedPluginsList } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useBatchSelection } from '@/composables/useBatchSelection'
import { useAutoSetup } from '@/composables/useAutoSetup'
import { usePluginFilter } from '@/composables/usePluginFilter'
import { usePluginUpdate } from '@/composables/usePluginUpdate'
import { useAssetLibrary } from '@/composables/useAssetLibrary'
import { isOnline as _isOnline } from '@/composables/useNetworkStatus'
import { usePluginStore, useSettingsStore } from '@/stores'
import { useContextMenu } from '@/composables/useContextMenu'
import type { ContextMenuEntry } from '@/composables/useContextMenu'
import { useFileManager } from '@/composables/useFileManager'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import ErrorState from '@/components/ErrorState.vue'
import AssetLibraryTab from '@/components/AssetLibraryTab.vue'
import GlobalUpgradeDialog from '@/components/GlobalUpgradeDialog.vue'

const pluginStore = usePluginStore()
const settingsStore = useSettingsStore()
const route = useRoute()
const router = useRouter()

const toast = useToast()
const { isRunning: isAutoSetupRunning, stepMessage: autoSetupMessage } = useAutoSetup()
const { t } = useI18n()

const plugins = computed(() => pluginStore.plugins)
const autoApplyEnabled = computed(() => settingsStore.settings.auto_apply ?? false)
const goToAutoApplySettings = () => router.push('/settings')

const loadAddonBackups = async () => {
  if (!selectedLinkId.value) return
  try {
    addonBackups.value = await api.listAddonBackups(selectedLinkId.value)
    showRollbackDialog.value = true
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const doRestoreAddonBackup = async (backupFile: string) => {
  if (!selectedLinkId.value) return
  isRestoringAddon.value = true
  try {
    await api.restoreAddonBackup(selectedLinkId.value, backupFile)
    toast.success(t('plugins.restoreSuccess'))
    showRollbackDialog.value = false
    showLinkerApplyResult.value = false
  } catch (error) {
    toast.error(t('plugins.restoreFailed'))
  } finally {
    isRestoringAddon.value = false
  }
}

const isLoading = ref(false)
const isRefreshing = ref(false)
const loadError = ref<string | null>(null)
const hasLoaded = ref(false)
let unlistenAutoSetup: UnlistenFn | null = null
const remoteUrl = ref('')
const remoteGitRef = ref('')
const showRemoteDialog = ref(false)
const gitRefs = ref<Array<{ name: string; ref_type: string }>>([])
const isLoadingGitRefs = ref(false)
const gitRefDecisionMade = ref(false)
const showPluginDetail = ref(false)
const selectedPlugin = ref<Plugin | null>(null)
const pluginDependencies = ref<PluginDependency[]>([])
const pluginStorageStats = ref<PluginStorageStats | null>(null)
const pluginBindings = ref<ProjectBinding[]>([])
const pluginBindingCountMap = ref<Map<string, number>>(new Map())
const showImportModeDialog = ref(false)
const importMode = ref<'copy' | 'move' | 'reference'>('copy')
const totalStorageStats = ref<TotalStorageStats | null>(null)

// ─── Global Upgrade ───
const showGlobalUpgradeDialog = ref(false)
const globalUpgradePluginId = ref('')
const globalUpgradePluginName = ref('')

const openGlobalUpgradeDialog = (plugin: Plugin) => {
  globalUpgradePluginId.value = plugin.plugin_id
  globalUpgradePluginName.value = plugin.name
  showGlobalUpgradeDialog.value = true
}

const showAddMenu = ref(false)
const isDragOver = ref(false)
const dragCounter = ref(0)

const activeTab = ref<'repository' | 'bindings' | 'assetLibrary'>('repository')

const featuredPlugins = ref<FeaturedPluginsList | null>(null)
const showFeatured = ref(true)

const pluginContextMenu = useContextMenu()
const { openInFileManager: openPluginInFileManager } = useFileManager()

const showPluginContextMenu = (event: MouseEvent, plugin: Plugin) => {
  pluginContextMenu.show(event, [
    {
      label: t('plugins.contextMenu.viewDetails'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>',
      action: () => { selectedPlugin.value = plugin; showPluginDetail.value = true },
    },
    { separator: true },
    {
      label: t('plugins.contextMenu.bindToProject'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>',
      action: () => { quickBindPlugin.value = plugin; showQuickBindDialog.value = true },
    },
    {
      label: t('plugins.contextMenu.globalUpgrade'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" /></svg>',
      action: () => openGlobalUpgradeDialog(plugin),
    },
    { separator: true },
    {
      label: t('plugins.contextMenu.openInFileManager'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>',
      action: () => {
        if (plugin.versions[0]?.units[0]?.install_path) {
          openPluginInFileManager(plugin.versions[0].units[0].install_path)
        }
      },
    },
    {
      label: t('plugins.contextMenu.delete'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>',
      action: async () => {
        try {
          const bindings = await api.getPluginBindings(plugin.plugin_id)
          for (const b of bindings) {
            await api.unbindPlugin(b.project_id, plugin.plugin_id).catch(() => {})
          }
          await api.removePlugin(plugin.plugin_id)
          toast.success(t('common.projectDeleted'))
          await loadPlugins(true)
          loadPluginBindingCounts()
        } catch (error) {
          toast.error(String(error))
        }
      },
      danger: true,
    },
  ] as ContextMenuEntry[])
}

async function loadFeaturedPlugins() {
  try {
    featuredPlugins.value = await api.getFeaturedPlugins()
  } catch {
    featuredPlugins.value = null
  }
}

async function importFeaturedPlugin(sourceUrl: string) {
  showRemoteDialog.value = true
  gitRefDecisionMade.value = false
  remoteGitRef.value = ''
  gitRefs.value = []
  await nextTick()
  remoteUrl.value = sourceUrl
  onRemoteUrlChange()
}

const showQuickBindDialog = ref(false)
const quickBindPlugin = ref<Plugin | null>(null)
const quickBindProjects = ref<Project[]>([])
const quickBindSelectedProjectIds = ref<Set<string>>(new Set())
const quickBindBoundProjectIds = ref<Set<string>>(new Set())
const quickBindVersionIdx = ref(0)
const quickBindUnitIdx = ref(0)
const isQuickBinding = ref(false)

const showVersionSwitchDialog = ref(false)
const versionSwitchBinding = ref<ProjectBinding | null>(null)
const versionSwitchPlugin = ref<Plugin | null>(null)
const versionSwitchVersionIdx = ref(0)
const versionSwitchUnitIdx = ref(0)
const isSwitchingVersion = ref(false)

const batchProgress = ref<{ current: number; total: number; message: string } | null>(null)
const batchFailedItems = ref<{ id: string; name: string; error: string }[]>([])
const mountStrategyDisplay = ref<string>('')

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
const showLinkerUnbindConfirm = ref(false)
const pendingUnbindBinding = ref<ProjectBinding | null>(null)
const versionSelectPlugin = ref<Plugin | null>(null)
const selectedVersionIdx = ref(0)
const selectedUnitIdx = ref(0)
const isLinkerBatchBinding = ref(false)
const isLinkerBatchUnbinding = ref(false)
const isLinkerBatchApplying = ref(false)
const linkerApplyResult = ref<any>(null)
const showLinkerApplyResult = ref(false)
const showLinkerBatchApplyResult = ref(false)
const batchApplyResults = ref<any[]>([])
const showGraphView = ref(false)
const linkerSearchQuery = ref('')
const linkerProjectBindingCounts = ref<Map<string, number>>(new Map())
const linkerProjectBindingNames = ref<Map<string, string[]>>(new Map())
const harborConfigStatus = ref<Map<string, boolean>>(new Map())
const addonBackups = ref<any[]>([])
const showRollbackDialog = ref(false)
const isRestoringAddon = ref(false)

const showHarborConfigDialog = ref(false)
const showDeleteConfigConfirm = ref(false)
const showCleanupConfirm = ref(false)
const isExportingConfig = ref(false)
const exportSkippedLocal = ref<string[]>([])
const isSyncingConfig = ref(false)
const harborConfigContent = ref<string | null>(null)
const syncResult = ref<{ imported: number; bound: number; skipped: number; errors: string[] } | null>(null)
const showSyncResultDialog = ref(false)

const showUidConflictDialog = ref(false)
const uidConflicts = ref<{ plugin_id: string; plugin_name: string; conflicting_uids: string[] }[]>([])
const pendingBindAfterUidCheck = ref<(() => void) | null>(null)

const handleBindWithCopyMode = async () => {
  showUidConflictDialog.value = false
  if (mountStrategyDisplay.value !== 'Copy') {
    try {
      const settings = await api.getSettings()
      settings.mount_strategy = 'Copy'
      await api.saveSettings(settings)
      mountStrategyDisplay.value = 'Copy'
      toast.success(t('settings.saved'))
    } catch (error) {
      toast.error(t('common.operationFailed', { error: String(error) }))
      return
    }
  }
  pendingBindAfterUidCheck.value?.()
}

const {
  searchQuery,
  filterCompatibility,
  filterSource,
  showOnlyDuplicates,
  showFavoritesOnly,
  filteredPlugins,
  displayedPlugins,
  hasMorePlugins,
  loadMorePlugins,
  favoritePlugins,
  checkAndShowDuplicates,
} = usePluginFilter(plugins)

const batchRemovePlugins = async () => {
  const ids = Array.from(selectedPluginIds.value)
  if (ids.length === 0) return
  showBatchDeleteConfirm.value = true
}

const onBatchDeleteConfirm = async () => {
  const ids = Array.from(selectedPluginIds.value)
  try {
    const bindingResults = await Promise.allSettled(
      ids.map(id => api.getPluginBindings(id))
    )
    const affectedProjectIds = new Set<string>()
    const unbindPromises: Promise<void>[] = []
    bindingResults.forEach((result, i) => {
      if (result.status === 'fulfilled') {
        for (const b of result.value) {
          affectedProjectIds.add(b.project_id)
          unbindPromises.push(api.unbindPlugin(b.project_id, ids[i]).catch(() => {}))
        }
      }
    })
    await Promise.allSettled(unbindPromises)
    const result = await api.batchRemovePlugins(ids)
    await Promise.allSettled(
      Array.from(affectedProjectIds).map(id => api.applyChanges(id))
    )
    if (result.failed_count > 0) {
      toast.warning(t('common.batchDeleteComplete', { success: result.success_count, failed: result.failed_count }))
    } else {
      toast.success(t('common.batchDeleteSuccess', { count: result.success_count }))
    }
    clearPluginSelection()
    await loadPlugins(true)
    loadPluginBindingCounts()
  } catch (error) {
    toast.error(t('common.batchDeleteFailed', { error }))
  }
}

const showBatchDeleteConfirm = ref(false)

const {
  selectedIds: selectedPluginIds,
  isBatchMode,
  selectedCount: selectedPluginCount,
  toggleSelection: togglePluginSelection,
  selectAll: selectAllPlugins,
  clearSelection: clearPluginSelection,
} = useBatchSelection<Plugin>({
  items: filteredPlugins,
  getId: (p) => p.plugin_id,
})

const loadPlugins = async (force = false) => {
  if (!force && hasLoaded.value && pluginStore.plugins.length > 0) {
    isRefreshing.value = true
    try {
      await pluginStore.loadPlugins()
      loadPluginBindingCounts().catch(() => {})
    } catch (error) {
      loadError.value = String(error)
    } finally {
      isRefreshing.value = false
    }
    return
  }
  isLoading.value = true
  loadError.value = null
  try {
    await pluginStore.loadPlugins()
    loadPluginBindingCounts().catch(() => {})
  } catch (error) {
    loadError.value = String(error)
  } finally {
    isLoading.value = false
    hasLoaded.value = true
  }
}

const loadPluginBindingCounts = async () => {
  try {
    const projects = await api.getProjects()
    const countMap = new Map<string, number>()
    const batchSize = 5
    for (let i = 0; i < projects.length; i += batchSize) {
      const batch = projects.slice(i, i + batchSize)
      const results = await Promise.allSettled(
        batch.map(project => api.getProjectBindings(project.project_id))
      )
      results.forEach((result) => {
        if (result.status === 'fulfilled') {
          for (const binding of result.value) {
            countMap.set(binding.plugin_id, (countMap.get(binding.plugin_id) || 0) + 1)
          }
        }
      })
    }
    pluginBindingCountMap.value = countMap
  } catch {
    // ignore binding count load errors
  }
}

const handleDragEnter = (e: DragEvent) => {
  e.preventDefault()
  dragCounter.value++
  isDragOver.value = true
}

const handleDragLeave = (e: DragEvent) => {
  e.preventDefault()
  dragCounter.value--
  if (dragCounter.value <= 0) {
    isDragOver.value = false
    dragCounter.value = 0
  }
}

const handleDragOver = (e: DragEvent) => {
  e.preventDefault()
}

const handleDrop = async (e: DragEvent) => {
  e.preventDefault()
  isDragOver.value = false
  dragCounter.value = 0

  if (activeTab.value !== 'repository') return

  const files = e.dataTransfer?.files
  if (!files || files.length === 0) return

  const paths: string[] = []
  for (let i = 0; i < files.length; i++) {
    const path = (files[i] as any).path as string | undefined
    if (path) paths.push(path)
  }
  if (paths.length === 0) return

  isLoading.value = true
  try {
    let imported = 0
    for (const path of paths) {
      try {
        const result = await api.importPluginFromLocal(path)
        imported++
        if (imported === 1) showPostImportGuide(result.name, result)
      } catch (error) {
        toast.error(t('common.loadFailed', { error: String(error) }))
      }
    }
    if (imported > 0) {
      toast.success(t('plugins.importPluginSuccess', { name: `${imported}` }))
      await loadPlugins(true)
    }
  } finally {
    isLoading.value = false
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
              showPostImportGuide(result.name, result)
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
      showPostImportGuide(result.name, result)
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
      const dirPath = selected.substring(0, Math.max(selected.lastIndexOf('/'), selected.lastIndexOf('\\')))
      const result = await api.importPluginFromLocal(dirPath || selected)
      toast.success(t('plugins.importPluginSuccess', { name: result.name }))
      await loadPlugins(true)
      showPostImportGuide(result.name, result)
    }
  } catch (error) {
    toast.error(t('common.addProjectFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const fetchGitRefs = async () => {
  const url = remoteUrl.value.trim()
  if (!url || (!url.endsWith('.git') && !url.includes('github.com'))) {
    gitRefs.value = []
    return
  }
  isLoadingGitRefs.value = true
  try {
    gitRefs.value = await api.listGitRefs(url)
  } catch {
    gitRefs.value = []
  } finally {
    isLoadingGitRefs.value = false
  }
}

let gitRefsDebounce: ReturnType<typeof setTimeout> | null = null
const onRemoteUrlChange = () => {
  if (gitRefsDebounce) clearTimeout(gitRefsDebounce)
  gitRefDecisionMade.value = false
  remoteGitRef.value = ''
  gitRefsDebounce = setTimeout(fetchGitRefs, 800)
}

const importFromRemote = async () => {
  if (!remoteUrl.value) {
    toast.warning(t('plugins.enterRemoteUrl'))
    return
  }
  isLoading.value = true
  try {
    const url = remoteUrl.value.trim()
    const isGitUrl = url.endsWith('.git') || (url.includes('github.com') && !url.includes('/archive/') && !url.endsWith('.zip') && !url.endsWith('.tar.gz'))
    const result = isGitUrl
      ? await api.importPluginFromGit(url, remoteGitRef.value.trim() || undefined)
      : await api.importPluginFromUrl(url)
    toast.success(t('plugins.importPluginSuccess', { name: result.name }))
    remoteUrl.value = ''
    remoteGitRef.value = ''
    gitRefDecisionMade.value = false
    showRemoteDialog.value = false
    await loadPlugins(true)
    showPostImportGuide(result.name, result)
  } catch (error) {
    toast.error(t('common.addProjectFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const addMenuRef = ref<HTMLElement | null>(null)

const handleClickOutside = (event: MouseEvent) => {
  if (showAddMenu.value && addMenuRef.value && !addMenuRef.value.contains(event.target as Node)) {
    showAddMenu.value = false
  }
}

const searchInputRef = ref<HTMLInputElement | null>(null)
const handleCtrlF = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
    e.preventDefault()
    searchInputRef.value?.focus()
    searchInputRef.value?.select()
  }
}

onMounted(async () => {
  loadPlugins()
  loadTotalStorageStats()
  loadFeaturedPlugins()
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleCtrlF)

  unlistenAutoSetup = await listen('auto-setup-complete', () => {
    loadPlugins()
    loadTotalStorageStats()
  })

  if (route.query.tab === 'bindings') {
    activeTab.value = 'bindings'
    await loadLinkerData()
    if (route.query.project && typeof route.query.project === 'string') {
      selectedLinkId.value = route.query.project
      selectedLinkProjectIds.value = new Set([route.query.project])
      await loadLinkerBindings(route.query.project)
    }
  }

  if (route.query.bindProject && typeof route.query.bindProject === 'string') {
    const projectId = route.query.bindProject
    if (plugins.value.length > 0) {
      const firstPlugin = plugins.value[0]
      quickBindPlugin.value = firstPlugin
      quickBindVersionIdx.value = 0
      quickBindUnitIdx.value = 0
      quickBindSelectedProjectIds.value = new Set([projectId])
      try {
        quickBindProjects.value = await api.getProjects()
      } catch {
        quickBindProjects.value = []
      }
      showPluginDetail.value = false
    showQuickBindDialog.value = true
    }
  }
  if (route.query.action === 'import') {
    await nextTick()
    showImportModeDialog.value = true
    router.replace({ path: '/plugins' })
  }
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleCtrlF)
  if (unlistenAutoSetup) {
    unlistenAutoSetup()
  }
})

const showDeletePluginConfirm = ref(false)
const deletePluginId = ref('')

const showVersionDeleteConfirm = ref(false)
const versionDeletePluginId = ref('')
const versionDeleteVersionId = ref('')
const versionDeleteWarning = ref('')

const showDuplicateConfirm = ref(false)
const duplicateCheckResult = ref<DuplicateCheckResult | null>(null)
const pendingImportAction = ref<(() => Promise<void>) | null>(null)

useDialogEscape(showRemoteDialog)
useDialogEscape(showPluginDetail)
useDialogEscape(showImportModeDialog)
useDialogEscape(showDuplicateConfirm)

const getMountPath = (unit: { subdirectory?: string; name: string; dir_name?: string }, plugin?: Plugin) => {
  let folderName = unit.dir_name || ''
  if (!folderName || folderName === 'payload') {
    if (unit.subdirectory) {
      const parts = unit.subdirectory.replace(/\\/g, '/').split('/')
      folderName = parts[parts.length - 1] || ''
    }
  }
  if (!folderName || folderName === 'payload') {
    folderName = plugin?.name || unit.name || 'plugin'
  }
  const isAssetPack = plugin?.asset_type === 'AssetPack'
  return isAssetPack ? `assets/${folderName}` : `addons/${folderName}`
}

const isCompatWarning = (plugin: Plugin, project: Project) => {
  if (plugin.compatibility === 'Both' || plugin.compatibility === 'Unknown') return false
  const projectMajor = project.godot_version?.startsWith('4') ? '4' : project.godot_version?.startsWith('3') ? '3' : null
  if (!projectMajor) return false
  if (plugin.compatibility === 'Godot4' && projectMajor !== '4') return true
  if (plugin.compatibility === 'Godot3' && projectMajor !== '3') return true
  return false
}

const getBindingVersion = (binding: ProjectBinding) => {
  const plugin = plugins.value.find(p => p.plugin_id === binding.plugin_id)
  if (!plugin) return null
  const version = plugin.versions.find(v => v.version_id === binding.version_id)
  return version?.version || null
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
      await Promise.allSettled(
        deletePluginBindings.value.map(binding =>
          api.unbindPlugin(binding.project_id, deletePluginId.value)
        )
      )
    }
    await api.removePlugin(deletePluginId.value)
    selectedPluginIds.value.delete(deletePluginId.value)
    selectedPluginIds.value = new Set(selectedPluginIds.value)
    if (selectedPluginIds.value.size === 0) {
      isBatchMode.value = false
    }
    toast.success(t('common.projectDeleted'))
    showPluginDetail.value = false
    selectedPlugin.value = null
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

const loadTotalStorageStats = async () => {
  try {
    totalStorageStats.value = await api.getTotalStorageStats()
  } catch (e) {
    console.error('Failed to load total storage stats:', e)
  }
}

const cleanupOrphaned = async () => {
  showCleanupConfirm.value = true
}

const onConfirmCleanup = async () => {
  showCleanupConfirm.value = false
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

const showPostImportGuide = async (pluginName: string, plugin?: Plugin) => {
  if (plugin) {
    quickBindPlugin.value = plugin
    quickBindVersionIdx.value = 0
    quickBindUnitIdx.value = 0
    quickBindSelectedProjectIds.value = new Set()
    try {
      quickBindProjects.value = await api.getProjects()
    } catch {
      quickBindProjects.value = []
    }
    try {
      const bindings = await api.getPluginBindings(plugin.plugin_id)
      quickBindBoundProjectIds.value = new Set(bindings.map(b => b.project_id))
    } catch {
      quickBindBoundProjectIds.value = new Set()
    }
    showQuickBindDialog.value = true
  } else {
    setTimeout(() => {
      toast.info(t('plugins.postImportGuide', { name: pluginName }))
    }, 800)
  }
}

const {
  showUpdatesDialog,
  pluginUpdates,
  isCheckingUpdates,
  isBatchUpdating,
  expandedReleaseNotes,
  updatablePluginIds,
  checkPluginUpdates,
  updateGitPlugin,
  batchUpdatePlugins,
} = usePluginUpdate({ loadPlugins })

const {
  openAssetLibraryTab,
} = useAssetLibrary({
  activeTab,
  loadPlugins,
  showPostImportGuide,
})

useDialogEscape(showUpdatesDialog)

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
    const unchecked = pluginBindings.value.filter(b => b.is_healthy === undefined)
    if (unchecked.length > 0) {
      const healthResults = await Promise.allSettled(
        unchecked.map(b => api.checkBindingHealth(b.project_id))
      )
      healthResults.forEach((result, i) => {
        if (result.status === 'fulfilled') {
          const healthBinding = result.value.find((hb: ProjectBinding) => hb.plugin_id === unchecked[i].plugin_id)
          if (healthBinding) {
            unchecked[i].is_healthy = healthBinding.is_healthy
          }
        }
      })
    }
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
    const bindings = await api.getPluginBindings(pluginId)
    const affectedBindings = bindings.filter(b => b.version_id === versionId)
    if (affectedBindings.length > 0) {
      const projectNames = affectedBindings.map(b => b.project_id).filter((v, i, a) => a.indexOf(v) === i)
      versionDeletePluginId.value = pluginId
      versionDeleteVersionId.value = versionId
      versionDeleteWarning.value = t('plugins.versionDeleteBindingWarning', { count: affectedBindings.length, projects: projectNames.length })
      showPluginDetail.value = false
      showVersionDeleteConfirm.value = true
      return
    }
    versionDeletePluginId.value = pluginId
    versionDeleteVersionId.value = versionId
    versionDeleteWarning.value = ''
    showVersionDeleteConfirm.value = true
  } catch (error) {
    toast.error(t('common.deleteFailed', { error }))
  }
}

const onVersionDeleteConfirm = async () => {
  const pluginId = versionDeletePluginId.value
  const versionId = versionDeleteVersionId.value
  try {
    const bindings = await api.getPluginBindings(pluginId)
    const affectedBindings = bindings.filter(b => b.version_id === versionId)
    await Promise.allSettled(
      affectedBindings.map(b =>
        api.unbindPlugin(b.project_id, b.plugin_id)
          .catch(() => {})
      )
    )
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
    try {
      await api.applyChanges(projectId)
    } catch {
      // ignore apply errors
    }
    toast.success(t('plugins.bindDialog.repairSuccess'))
    if (selectedPlugin.value) {
      pluginBindings.value = await api.getPluginBindings(selectedPlugin.value.plugin_id)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const unbindFromDetail = async (binding: ProjectBinding) => {
  try {
    await api.unbindPlugin(binding.project_id, binding.plugin_id)
    try {
      await api.applyChanges(binding.project_id)
    } catch {
      // ignore apply errors
    }
    toast.success(t('linker.pluginUnbound'))
    if (selectedPlugin.value) {
      pluginBindings.value = await api.getPluginBindings(selectedPlugin.value.plugin_id)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const installedPluginIds = computed(() => new Set(plugins.value.map(p => p.plugin_id)))

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
  let skippedCount = 0
  for (const depId of missingDepPluginIds.value) {
    const dep = pluginDependencies.value.find(d => d.plugin_id === depId)
    if (!dep) {
      failCount++
      continue
    }
    const isGitUrl = dep.version_constraint && (
      dep.version_constraint.startsWith('https://') ||
      dep.version_constraint.startsWith('http://') ||
      dep.version_constraint.startsWith('git@') ||
      dep.version_constraint.endsWith('.git')
    )
    if (isGitUrl) {
      try {
        await api.importPluginFromGit(dep.version_constraint)
        successCount++
      } catch {
        failCount++
      }
    } else if (dep.version_constraint) {
      try {
        const searchResult = await api.searchAssetLibrary({ filter: dep.version_constraint, page: 0, max_results: 5 })
        const matchAsset = searchResult.result?.find((a: any) =>
          a.title.toLowerCase() === dep.plugin_id.toLowerCase() ||
          a.title.toLowerCase().includes(dep.plugin_id.toLowerCase())
        )
        if (matchAsset) {
          await api.importFromAssetLibraryWithProgress(String(matchAsset.asset_id))
          successCount++
        } else {
          skippedCount++
        }
      } catch {
        skippedCount++
      }
    } else {
      skippedCount++
    }
  }
  isInstallingDeps.value = false
  if (skippedCount > 0) {
    toast.warning(t('plugins.depDialog.partialSuccess', { success: successCount, failed: failCount + skippedCount }))
  } else if (failCount > 0) {
    toast.warning(t('plugins.depDialog.partialSuccess', { success: successCount, failed: failCount }))
  } else {
    toast.success(t('plugins.depDialog.success', { count: successCount }))
  }
  await loadPlugins(true)
  if (selectedPlugin.value) {
    pluginDependencies.value = await api.resolvePluginDependencies(selectedPlugin.value.plugin_id)
  }
}

const loadLinkerData = async () => {
  try {
    const [projs] = await Promise.all([
      api.getProjects(),
      api.getPlugins()
    ])
    linkerProjects.value = projs
    const countMap = new Map<string, number>()
    const namesMap = new Map<string, string[]>()
    const bindingResults = await Promise.allSettled(
      projs.map(project => api.getProjectBindings(project.project_id))
    )
    bindingResults.forEach((result, i) => {
      const bindings = result.status === 'fulfilled' ? result.value : []
      countMap.set(projs[i].project_id, bindings.length)
      const names = bindings.slice(0, 3).map(b => {
        const plugin = plugins.value.find(p => p.plugin_id === b.plugin_id)
        return plugin?.name || b.plugin_id
      })
      namesMap.set(projs[i].project_id, names)
    })
    linkerProjectBindingCounts.value = countMap
    linkerProjectBindingNames.value = namesMap
    try {
      const ids = projs.map(p => p.project_id)
      const statusMap = await api.checkHarborConfigs(ids)
      const hMap = new Map<string, boolean>()
      for (const [k, v] of Object.entries(statusMap)) {
        hMap.set(k, v)
      }
      harborConfigStatus.value = hMap
    } catch {}
    if (!hasLoaded.value) {
      await loadPlugins(true)
    }
    try {
      const settings = await api.getSettings()
      mountStrategyDisplay.value = settings.mount_strategy || 'Symlink'
    } catch {}
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const loadLinkerBindings = async (projectId: string) => {
  try {
    linkerBindings.value = await api.getProjectBindings(projectId)
    const unchecked = linkerBindings.value.filter(b => b.is_healthy === undefined)
    if (unchecked.length > 0) {
      const healthResults = await Promise.allSettled(
        unchecked.map(b => api.checkBindingHealth(b.project_id))
      )
      healthResults.forEach((result, i) => {
        if (result.status === 'fulfilled') {
          const healthBinding = result.value.find((hb: ProjectBinding) => hb.plugin_id === unchecked[i].plugin_id)
          if (healthBinding) {
            unchecked[i].is_healthy = healthBinding.is_healthy
          }
        }
      })
    }
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
    await loadAllSelectedBindings()
  } else {
    selectedLinkId.value = project.project_id
    selectedLinkProjectIds.value = new Set([project.project_id])
    await loadLinkerBindings(project.project_id)
  }
}

const loadAllSelectedBindings = async () => {
  const projectIds = Array.from(selectedLinkProjectIds.value)
  const results = await Promise.allSettled(
    projectIds.map(id => api.getProjectBindings(id))
  )
  const allBindings: ProjectBinding[] = []
  results.forEach(result => {
    if (result.status === 'fulfilled') {
      allBindings.push(...result.value)
    }
  })
  linkerBindings.value = allBindings
}

const exportHarborConfig = async () => {
  if (!selectedLinkId.value) {
    toast.warning(t('linker.selectProject'))
    return
  }
  isExportingConfig.value = true
  try {
    const result = await api.writeHarborConfig(selectedLinkId.value)
    exportSkippedLocal.value = result.skipped_local || []
    const config = await api.readHarborConfigRaw(selectedLinkId.value)
    harborConfigContent.value = config || null
    harborConfigStatus.value.set(selectedLinkId.value, true)
    harborConfigStatus.value = new Map(harborConfigStatus.value)
    showHarborConfigDialog.value = true
    toast.success(t('linker.configExported'))
  } catch (error) {
    toast.error(t('linker.configExportFailed', { error: error instanceof Error ? error.message : String(error) }))
  } finally {
    isExportingConfig.value = false
  }
}

const deleteHarborConfig = async () => {
  if (!selectedLinkId.value) return
  showDeleteConfigConfirm.value = true
}

const onConfirmDeleteConfig = async () => {
  if (!selectedLinkId.value) return
  try {
    await api.deleteHarborConfig(selectedLinkId.value)
    harborConfigStatus.value.set(selectedLinkId.value!, false)
    harborConfigStatus.value = new Map(harborConfigStatus.value)
    showHarborConfigDialog.value = false
    harborConfigContent.value = null
    toast.success(t('linker.configDeleted'))
  } catch (error) {
    toast.error(t('linker.configDeleteFailed', { error: error instanceof Error ? error.message : String(error) }))
  }
}

const syncHarborConfig = async () => {
  if (!selectedLinkId.value) {
    toast.warning(t('linker.selectProject'))
    return
  }
  isSyncingConfig.value = true
  try {
    const result = await api.syncHarborConfig(selectedLinkId.value)
    syncResult.value = result
    if (result.imported === 0 && result.bound === 0 && result.skipped === 0 && result.errors.length === 0) {
      toast.info(t('linker.configAlreadyInSync'))
    } else {
      showSyncResultDialog.value = true
    }
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
    if (result.imported > 0 || result.bound > 0) {
      await api.applyChanges(selectedLinkId.value)
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error)
    if (errMsg.includes('.harbor.yml') || errMsg.includes('未找到')) {
      toast.info(t('linker.noHarborConfigHint'))
    } else {
      toast.error(t('linker.configSyncFailed', { error: errMsg }))
    }
  } finally {
    isSyncingConfig.value = false
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
  const mountPath = getMountPath(unit, plugin)
  const subdirectory = unit.subdirectory || ''
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

  if (selectedLinkProjectIds.value.size > 0) {
    const firstProjectId = Array.from(selectedLinkProjectIds.value)[0]
    try {
      const conflicts = await api.checkUidConflicts(firstProjectId, plugin.plugin_id)
      if (conflicts.length > 0) {
        uidConflicts.value = conflicts
        pendingBindAfterUidCheck.value = () => proceedBindPlugin(plugin, version, unit, mountPath, subdirectory)
        showUidConflictDialog.value = true
        return
      }
    } catch {}
  }

  await proceedBindPlugin(plugin, version, unit, mountPath, subdirectory)
}

const proceedBindPlugin = async (plugin: Plugin, version: any, unit: any, mountPath: string, subdirectory: string) => {
  try {
    await Promise.allSettled(
      Array.from(selectedLinkProjectIds.value).map(projectId =>
        api.bindPlugin(projectId, plugin.plugin_id, version.version_id, unit.unit_id, mountPath, subdirectory)
      )
    )
    const applyResults = await Promise.allSettled(
      Array.from(selectedLinkProjectIds.value).map(projectId =>
        api.applyChanges(projectId)
      )
    )
    const failedApplies = applyResults
      .map((r, i) => {
        if (r.status === 'fulfilled' && !r.value.success) {
          return `${Array.from(selectedLinkProjectIds.value)[i]}: ${r.value.errors.join(', ')}`
        }
        if (r.status === 'rejected') {
          return `${Array.from(selectedLinkProjectIds.value)[i]}: ${r.reason}`
        }
        return null
      })
      .filter(Boolean)
    if (failedApplies.length > 0) {
      toast.warning(t('linker.bindingApplyFailed', { errors: failedApplies.join('; ') }))
    } else {
      toast.success(t('linker.pluginBound', { name: plugin.name, version: version.version }))
      Promise.allSettled(
        Array.from(selectedLinkProjectIds.value).map(projectId =>
          api.enablePluginInProject(projectId, plugin.plugin_id)
        )
      ).catch(() => {})
    }
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
    if (linkerProjectBindingCounts.value.size > 0) {
      const countResults = await Promise.allSettled(
        Array.from(selectedLinkProjectIds.value).map(projectId =>
          api.getProjectBindings(projectId)
        )
      )
      countResults.forEach((result, i) => {
        const projectId = Array.from(selectedLinkProjectIds.value)[i]
        if (result.status === 'fulfilled') {
          linkerProjectBindingCounts.value.set(projectId, result.value.length)
          const names = result.value.slice(0, 3).map((b: ProjectBinding) => {
            const plugin = plugins.value.find(p => p.plugin_id === b.plugin_id)
            return plugin?.name || b.plugin_id
          })
          linkerProjectBindingNames.value.set(projectId, names)
        }
      })
      linkerProjectBindingCounts.value = new Map(linkerProjectBindingCounts.value)
      linkerProjectBindingNames.value = new Map(linkerProjectBindingNames.value)
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

const unbindPluginFromProject = (binding: ProjectBinding) => {
  pendingUnbindBinding.value = binding
  showLinkerUnbindConfirm.value = true
}

const confirmUnbindPlugin = async () => {
  if (!pendingUnbindBinding.value) return
  const binding = pendingUnbindBinding.value
  showLinkerUnbindConfirm.value = false
  try {
    await api.unbindPlugin(binding.project_id, binding.plugin_id)
    try {
      await api.applyChanges(binding.project_id)
    } catch (applyErr) {
      toast.warning(t('linker.bindingApplyFailed', { errors: applyErr instanceof Error ? applyErr.message : String(applyErr) }))
    }
    toast.success(t('linker.pluginUnbound'))
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
    if (linkerProjectBindingCounts.value.has(binding.project_id)) {
      try {
        const bindings = await api.getProjectBindings(binding.project_id)
        linkerProjectBindingCounts.value.set(binding.project_id, bindings.length)
        const names = bindings.slice(0, 3).map((b: ProjectBinding) => {
          const plugin = plugins.value.find(p => p.plugin_id === b.plugin_id)
          return plugin?.name || b.plugin_id
        })
        linkerProjectBindingNames.value.set(binding.project_id, names)
        linkerProjectBindingCounts.value = new Map(linkerProjectBindingCounts.value)
        linkerProjectBindingNames.value = new Map(linkerProjectBindingNames.value)
      } catch {}
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    pendingUnbindBinding.value = null
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
  const needsVersionSelect = Array.from(selectedLinkPluginIds.value).some(id => {
    const plugin = plugins.value.find(p => p.plugin_id === id)
    return plugin && (plugin.versions.length > 1 || (plugin.versions.length > 0 && plugin.versions[0].units.length > 1))
  })
  if (needsVersionSelect) {
    batchVersionSelectIdx.value = 0
    showBatchVersionSelectDialog.value = true
    return
  }
  showLinkerBatchBindDialog.value = true
}

const batchVersionSelectMap = ref<Map<string, { versionIdx: number, unitIdx: number }>>(new Map())
const batchVersionSelectIdx = ref(0)
const showBatchVersionSelectDialog = ref(false)
const batchVersionSelectPluginIds = computed(() => {
  return Array.from(selectedLinkPluginIds.value).filter(id => {
    const plugin = plugins.value.find(p => p.plugin_id === id)
    return plugin && (plugin.versions.length > 1 || (plugin.versions.length > 0 && plugin.versions[0].units.length > 1))
  })
})

const confirmBatchVersionSelect = () => {
  showBatchVersionSelectDialog.value = false
  showLinkerBatchBindDialog.value = true
}

const confirmBatchBind = async () => {
  isLinkerBatchBinding.value = true
  const requests = []
  for (const projectId of selectedLinkProjectIds.value) {
    for (const pluginId of selectedLinkPluginIds.value) {
      const plugin = plugins.value.find(p => p.plugin_id === pluginId)
      if (plugin && plugin.versions.length > 0) {
        const custom = batchVersionSelectMap.value.get(pluginId)
        let versionIdx = 0
        let unitIdx = 0
        if (custom) {
          versionIdx = custom.versionIdx
          unitIdx = custom.unitIdx
        }
        const version = plugin.versions[versionIdx]
        const unit = version?.units[unitIdx]
        if (unit) {
          const mountPath = getMountPath(unit, plugin)
          const subdirectory = unit.subdirectory || ''
          requests.push({ project_id: projectId, plugin_id: pluginId, version_id: version.version_id, unit_id: unit.unit_id, mount_path: mountPath, subdirectory })
        }
      }
    }
  }
  for (const req of requests) {
    const conflict = linkerBindings.value.find(b => b.project_id === req.project_id && b.mount_path === req.mount_path && b.plugin_id !== req.plugin_id)
    if (conflict) {
      const conflictPlugin = plugins.value.find(p => p.plugin_id === conflict.plugin_id)
      const conflictProject = linkerProjects.value.find(p => p.project_id === req.project_id)
      toast.warning(t('linker.mountConflict', { path: req.mount_path, plugin: `${conflictPlugin?.name || conflict.plugin_id} (${conflictProject?.name || req.project_id})` }))
      isLinkerBatchBinding.value = false
      return
    }
  }
  try {
    batchProgress.value = { current: 0, total: requests.length, message: t('plugins.batchProgress.binding') }
    const result = await api.batchBindPlugins(requests)
    batchProgress.value = { current: requests.length, total: requests.length, message: t('plugins.batchProgress.applying') }
    await Promise.allSettled(
      Array.from(selectedLinkProjectIds.value).map(projectId => api.applyChanges(projectId))
    )
    batchProgress.value = null
    if (result.failed_count > 0) {
      batchFailedItems.value = result.errors.map((e: string, i: number) => ({ id: String(i), name: String(i), error: e }))
      toast.warning(t('common.batchDeleteComplete', { success: result.success_count, failed: result.failed_count }))
    } else {
      toast.success(t('plugins.bindDialog.bindAndApplySuccess', { count: result.success_count, name: '' }))
    }
    selectedLinkPluginIds.value = new Set()
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
    try {
      await api.applyChanges(selectedLinkId.value)
    } catch (applyErr) {
      toast.warning(t('linker.bindingApplyFailed', { errors: applyErr instanceof Error ? applyErr.message : String(applyErr) }))
    }
    if (result.failed_count > 0) {
      toast.warning(t('common.batchDeleteComplete', { success: result.success_count, failed: result.failed_count }))
    } else {
      toast.success(t('linker.pluginUnbound'))
    }
    selectedLinkPluginIds.value = new Set()
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
  const applyResults = await Promise.allSettled(
    projectIds.map(id => api.applyChanges(id))
  )
  applyResults.forEach((result, i) => {
    const projectId = projectIds[i]
    const project = linkerProjects.value.find(p => p.project_id === projectId)
    if (result.status === 'fulfilled') {
      batchApplyResults.value.push({ project_id: projectId, project_name: project?.name || projectId, ...result.value })
    } else {
      batchApplyResults.value.push({ project_id: projectId, project_name: project?.name || projectId, success: false, errors: [String(result.reason)], created: [], removed: [] })
    }
  })
  isLinkerBatchApplying.value = false
  showLinkerBatchApplyDialog.value = false
  showLinkerBatchApplyResult.value = true
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
  const boundProjectIds = new Set(linkerBindings.value.map(b => b.project_id))
  const boundPluginIds = new Set(linkerBindings.value.map(b => b.plugin_id))
  const projs = linkerProjects.value.filter(p => boundProjectIds.has(p.project_id))
  const plgs = plugins.value.filter(p => boundPluginIds.has(p.plugin_id))
  projs.forEach((p, i) => {
    nodes.push({ type: 'project', id: p.project_id, name: p.name, y: i * 44 + 50 })
  })
  plgs.forEach((p, i) => {
    nodes.push({ type: 'plugin', id: p.plugin_id, name: p.name, y: i * 44 + 50 })
  })
  return nodes
})

const graphSvgHeight = computed(() => {
  const boundProjectIds = new Set(linkerBindings.value.map(b => b.project_id))
  const boundPluginIds = new Set(linkerBindings.value.map(b => b.plugin_id))
  const projs = linkerProjects.value.filter(p => boundProjectIds.has(p.project_id))
  const plgs = plugins.value.filter(p => boundPluginIds.has(p.plugin_id))
  return Math.max(Math.max(projs.length, plgs.length) * 44 + 80, 200)
})

const graphLinks = computed(() => {
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
useDialogEscape(showLinkerUnbindConfirm)
useDialogEscape(showBatchVersionSelectDialog)
useDialogEscape(showLinkerBatchApplyDialog)
useDialogEscape(showLinkerVersionSelect)
useDialogEscape(showLinkerApplyResult)
useDialogEscape(showLinkerBatchApplyResult)
useDialogEscape(showQuickBindDialog)
useDialogEscape(showVersionSwitchDialog)
useDialogEscape(showVersionDeleteConfirm)
useDialogEscape(showRollbackDialog)
useDialogEscape(showHarborConfigDialog)
useDialogEscape(showSyncResultDialog)
useDialogEscape(showUidConflictDialog)

const doQuickBind = async () => {
  if (!quickBindPlugin.value) return
  if (quickBindSelectedProjectIds.value.size === 0) {
    toast.warning(t('plugins.quickBind.noSelection'))
    return
  }
  const plugin = quickBindPlugin.value
  if (plugin.versions.length === 0) {
    toast.warning(t('linker.noPluginVersions'))
    return
  }
  const version = plugin.versions[quickBindVersionIdx.value]
  const unit = version?.units[quickBindUnitIdx.value]
  if (!version || !unit) {
    toast.warning(t('linker.noPluginUnits'))
    return
  }
  isQuickBinding.value = true
  const mountPath = getMountPath(unit, plugin)
  const subdirectory = unit.subdirectory || ''
  let successCount = 0
  let failCount = 0
  const bindErrors: string[] = []
  const applyErrors: string[] = []
  for (const projectId of quickBindSelectedProjectIds.value) {
    try {
      await api.bindPlugin(projectId, plugin.plugin_id, version.version_id, unit.unit_id, mountPath, subdirectory)
      const applyResult = await api.applyChanges(projectId)
      if (!applyResult.success) {
        applyErrors.push(...applyResult.errors)
        failCount++
      } else {
        successCount++
        try {
          await api.enablePluginInProject(projectId, plugin.plugin_id)
        } catch {
          // ignore enable failure
        }
      }
    } catch (e: any) {
      bindErrors.push(e?.toString() || t('common.unknownError'))
      failCount++
    }
  }
  isQuickBinding.value = false
  showQuickBindDialog.value = false
  if (failCount > 0) {
    const allErrors = [...bindErrors, ...applyErrors]
    const errorMsg = allErrors.length > 0 ? allErrors.slice(0, 3).join('; ') + (allErrors.length > 3 ? ` ... (+${allErrors.length - 3})` : '') : ''
    toast.warning(t('plugins.bindDialog.partialSuccess', { success: successCount, failed: failCount }) + (errorMsg ? ` — ${errorMsg}` : ''))
  } else {
    toast.success(t('plugins.quickBind.bindSuccess', { name: plugin.name, count: successCount }))
  }
  await loadPlugins(true)
  loadPluginBindingCounts()
}

const closeQuickBind = () => {
  showQuickBindDialog.value = false
  quickBindPlugin.value = null
  quickBindSelectedProjectIds.value = new Set()
}

const toggleQuickBindProject = (projectId: string) => {
  const newSet = new Set(quickBindSelectedProjectIds.value)
  if (newSet.has(projectId)) {
    newSet.delete(projectId)
  } else {
    newSet.add(projectId)
  }
  quickBindSelectedProjectIds.value = newSet
}

const openVersionSwitch = (binding: ProjectBinding) => {
  const plugin = plugins.value.find(p => p.plugin_id === binding.plugin_id)
  if (!plugin) return
  versionSwitchBinding.value = binding
  versionSwitchPlugin.value = plugin
  const currentVersionIdx = plugin.versions.findIndex(v => v.version_id === binding.version_id)
  versionSwitchVersionIdx.value = currentVersionIdx >= 0 ? currentVersionIdx : 0
  versionSwitchUnitIdx.value = 0
  showVersionSwitchDialog.value = true
}

const doSwitchVersion = async () => {
  if (!versionSwitchBinding.value || !versionSwitchPlugin.value) return
  const binding = versionSwitchBinding.value
  const plugin = versionSwitchPlugin.value
  const version = plugin.versions[versionSwitchVersionIdx.value]
  const unit = version?.units[versionSwitchUnitIdx.value]
  if (!version || !unit) return
  isSwitchingVersion.value = true
  try {
    await api.unbindPlugin(binding.project_id, binding.plugin_id)
    const mountPath = getMountPath(unit, plugin)
    const subdirectory = unit.subdirectory || ''
    await api.bindPlugin(binding.project_id, plugin.plugin_id, version.version_id, unit.unit_id, mountPath, subdirectory)
    try {
      await api.applyChanges(binding.project_id)
    } catch {
      toast.warning(t('plugins.applyChangesFailed'))
    }
    toast.success(t('plugins.versionSwitch.success'))
    showVersionSwitchDialog.value = false
    if (selectedLinkId.value) {
      await loadLinkerBindings(selectedLinkId.value)
    }
  } catch (error) {
    toast.error(t('plugins.versionSwitch.failed', { error }))
  } finally {
    isSwitchingVersion.value = false
  }
}

const quickBindFromCard = async (plugin: Plugin) => {
  showPostImportGuide(plugin.name, plugin)
}

const removePluginAndReimport = async (pluginId: string) => {
  try {
    const bindings = await api.getPluginBindings(pluginId)
    await Promise.allSettled(
      bindings.map(b =>
        api.unbindPlugin(b.project_id, b.plugin_id)
          .catch(() => {})
      )
    )
    await api.removePlugin(pluginId)
    if (pendingImportAction.value) {
      pendingImportAction.value()
      pendingImportAction.value = null
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const retryBatchFailed = async () => {
  if (batchFailedItems.value.length === 0) return
  let successCount = 0
  let failCount = 0
  for (const item of batchFailedItems.value) {
    try {
      const plugin = plugins.value.find(p => p.plugin_id === item.id)
      if (plugin && plugin.versions.length > 0) {
        const version = plugin.versions[0]
        const unit = version.units[0]
        if (unit) {
          const mountPath = getMountPath(unit, plugin)
          const subdirectory = unit.subdirectory || ''
          for (const projectId of selectedLinkProjectIds.value) {
            await api.bindPlugin(projectId, plugin.plugin_id, version.version_id, unit.unit_id, mountPath, subdirectory)
            try { await api.applyChanges(projectId) } catch { toast.warning(t('plugins.applyChangesFailed')) }
          }
          successCount++
        }
      }
    } catch {
      failCount++
    }
  }
  if (failCount === 0) {
    toast.success(t('plugins.retrySuccess', { count: successCount }))
    batchFailedItems.value = []
  } else {
    toast.warning(t('plugins.retryFailedAgain', { count: failCount }))
  }
  if (selectedLinkId.value) {
    await loadLinkerBindings(selectedLinkId.value)
  }
}

</script>

<template>
  <div class="relative" @dragenter="handleDragEnter" @dragleave="handleDragLeave" @dragover="handleDragOver" @drop="handleDrop">
    <div v-if="isDragOver" class="fixed inset-0 bg-primary-500/10 border-2 border-dashed border-primary-500 z-50 flex items-center justify-center pointer-events-none">
      <div class="dialog-container text-center">
        <svg class="w-12 h-12 text-primary-500 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <p class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('plugins.dropToImport') }}</p>
      </div>
    </div>
    <div class="space-y-0">
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
      <div class="flex items-center gap-2">
        <h1 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('plugins.title') }}</h1>
        <div class="flex border-b border-gray-200/80 dark:border-surface-border/60">
          <button
            @click="activeTab = 'repository'"
            :class="['px-3 py-2 text-sm font-medium border-b-2 -mb-px transition-colors duration-150', activeTab === 'repository' ? 'border-primary-600 text-primary-600 dark:text-brand-primary' : 'border-transparent text-gray-500 hover:text-gray-700 dark:hover:text-gray-300']"
          >{{ t('plugins.tabRepository') }}</button>
          <button
            @click="activeTab = 'assetLibrary'; openAssetLibraryTab()"
            :class="['px-3 py-2 text-sm font-medium border-b-2 -mb-px transition-colors duration-150', activeTab === 'assetLibrary' ? 'border-primary-600 text-primary-600 dark:text-brand-primary' : 'border-transparent text-gray-500 hover:text-gray-700 dark:hover:text-gray-300']"
          >{{ t('assetLibrary.title') }}</button>
        </div>
      </div>
      <div v-if="activeTab === 'repository'" class="flex flex-wrap gap-2">
        <button
          @click="importFromProjects"
          :disabled="isLoading"
          class="btn-secondary disabled:opacity-50 text-sm"
        >
          {{ t('plugins.importFromProject.title') }}
        </button>
        <button
          @click="checkPluginUpdates"
          :disabled="isCheckingUpdates || isLoading"
          class="btn-secondary disabled:opacity-50 text-sm"
        >
          {{ isCheckingUpdates ? t('plugins.checkingUpdates') : t('plugins.checkUpdates') }}
        </button>
        <div class="relative" ref="addMenuRef">
          <button
            @click="showAddMenu = !showAddMenu"
            :disabled="isLoading"
            class="btn-primary disabled:opacity-50 text-sm flex items-center gap-1.5"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 4v16m8-8H4" />
            </svg>
            {{ t('plugins.addPlugin') }}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div v-if="showAddMenu" class="absolute right-0 mt-2 w-60 bg-white dark:bg-surface-card rounded-[6px] border border-gray-200/60 dark:border-surface-border/40 z-50 py-1">
            <div class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-content-muted uppercase tracking-wider">{{ t('plugins.addMenu.localLabel') }}</div>
            <button
              @click="importFromLocal(); showAddMenu = false"
              class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover flex items-center gap-2"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              <div>
                <div class="font-medium">{{ t('plugins.fromDir') }}</div>
                <div class="text-xs text-gray-500 dark:text-content-muted">{{ t('plugins.addMenu.fromDirDesc') }}</div>
              </div>
            </button>
            <button
              @click="importFromFile(); showAddMenu = false"
              class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover flex items-center gap-2"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <div>
                <div class="font-medium">{{ t('plugins.fromFile') }}</div>
                <div class="text-xs text-gray-500 dark:text-content-muted">{{ t('plugins.addMenu.fromFileDesc') }}</div>
              </div>
            </button>
            <div class="border-t border-gray-200/60 dark:border-surface-border/40 my-1"></div>
            <div class="px-3 py-1.5 text-xs font-medium text-gray-400 dark:text-content-muted uppercase tracking-wider">{{ t('plugins.addMenu.remoteLabel') }}</div>
            <button
              @click="showRemoteDialog = true; showAddMenu = false"
              class="w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover flex items-center gap-2"
            >
              <svg class="w-4 h-4 text-gray-500 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
              <div>
                <div class="font-medium">{{ t('plugins.fromRemote') }}</div>
                <div class="text-xs text-gray-500 dark:text-content-muted">{{ t('plugins.addMenu.fromRemoteDesc') }}</div>
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'repository'" class="border-b border-gray-200/60 dark:border-surface-border/40 pb-2 mt-3">
      <div class="flex flex-col lg:flex-row gap-2">
        <div class="flex-1">
          <input
            ref="searchInputRef"
            v-model="searchQuery"
            type="text"
            :placeholder="t('plugins.search')"
            class="input-field"
          />
        </div>
        <div class="flex flex-wrap gap-1.5 items-center">
          <select
            v-model="filterCompatibility"
            class="select-field"
          >
            <option value="all">{{ t('plugins.allVersions') }}</option>
            <option value="Godot4">Godot 4</option>
            <option value="Godot3">Godot 3</option>
            <option value="Both">{{ t('plugins.compat.both') }}</option>
          </select>
          <select
            v-model="filterSource"
            class="select-field"
          >
            <option value="all">{{ t('plugins.allSource') }}</option>
            <option value="Local">{{ t('plugins.source.local') }}</option>
            <option value="Git">{{ t('plugins.source.git') }}</option>
            <option value="AssetLibrary">{{ t('plugins.source.assetlibrary') }}</option>
          </select>
          <button
            @click="showFavoritesOnly = !showFavoritesOnly"
            :class="[
              'px-3 py-2 rounded-btn text-sm font-medium transition-colors',
              showFavoritesOnly
                ? 'filter-btn-active transition-colors'
                : 'filter-btn transition-colors'
            ]"
          >
            <span class="flex items-center gap-1">
              <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
              </svg>
              {{ favoritePlugins }} {{ t('plugins.favorites') }}
            </span>
          </button>
          <button
            v-if="showOnlyDuplicates"
            @click="showOnlyDuplicates = false"
            class="px-3 py-1.5 rounded-btn text-sm font-medium transition-colors bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400"
          >
            {{ t('plugins.showDuplicates') }} ✕
          </button>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'repository'">
    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <ErrorState
      v-else-if="loadError"
      :title="t('common.loadFailed', { error: '' })"
      :description="loadError"
      :retryLabel="t('common.retry')"
      @retry="loadPlugins(true)"
    />

    <div v-else-if="isAutoSetupRunning && filteredPlugins.length === 0 && plugins.length === 0" class="text-center py-16">
      <div class="animate-spin rounded-full h-10 w-10 border-2 border-primary-600 border-t-transparent mx-auto"></div>
      <h3 class="mt-4 text-sm font-medium text-gray-900 dark:text-content-primary">{{ autoSetupMessage }}</h3>
      <p class="mt-1 text-xs text-gray-500 dark:text-content-muted">{{ t('autoSetup.pleaseWait') }}</p>
    </div>

    <div v-else-if="filteredPlugins.length === 0 && plugins.length === 0" class="text-center py-12 max-w-md mx-auto">
      <EmptyState
        :title="t('plugins.onboarding.title')"
        :description="t('plugins.onboarding.desc')"
        :actionLabel="t('plugins.onboarding.fromDir')"
        @action="importFromLocal"
        :shortcuts="[
          { key: 'Ctrl+K', description: t('commandPalette.title') },
        ]"
      />

      <div v-if="featuredPlugins && featuredPlugins.plugins.length > 0 && showFeatured" class="mt-4 pt-3 border-t border-gray-200/60 dark:border-surface-border/40">
        <div class="flex items-center justify-between mb-2">
          <h4 class="text-sm font-semibold text-gray-900 dark:text-content-primary flex items-center gap-1.5">
            <svg class="w-4 h-4 text-amber-500" fill="currentColor" viewBox="0 0 20 20"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"/></svg>
            {{ t('plugins.featured.title') }}
          </h4>
          <button @click="showFeatured = false" class="text-gray-400 hover:text-gray-600 dark:hover:text-content-secondary">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12"/></svg>
          </button>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
          <div
            v-for="fp in featuredPlugins.plugins"
            :key="fp.source_url"
            class="flex items-start gap-2 p-2 bg-white dark:bg-surface-card border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] hover:border-primary-300 dark:hover:border-surface-border transition-colors cursor-pointer group"
            @click="importFeaturedPlugin(fp.source_url)"
          >
            <div class="w-7 h-7 bg-primary-100 dark:bg-surface-hover rounded-[4px] flex items-center justify-center flex-shrink-0">
              <svg class="w-4 h-4 text-primary-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-gray-900 dark:text-content-primary group-hover:text-primary-600 dark:group-hover:text-primary-400 truncate">{{ fp.name }}</div>
              <div class="text-xs text-gray-500 dark:text-content-muted truncate">{{ fp.description }}</div>
              <div class="flex items-center gap-2 mt-1">
                <span class="text-xs text-gray-400 dark:text-content-muted">{{ fp.author }}</span>
                <span v-if="fp.compatibility" class="text-xs px-1.5 py-0.5 rounded-[4px] bg-blue-100 dark:bg-surface-hover text-blue-700 dark:text-content-secondary">{{ fp.compatibility }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="mt-4 flex flex-wrap gap-2 justify-center">
        <div class="flex items-center gap-1.5 text-xs text-gray-400 dark:text-content-muted">
          <kbd class="px-1.5 py-0.5 rounded-[4px] bg-gray-100 dark:bg-surface-hover border border-gray-200/60 dark:border-surface-border/40 font-mono text-[11px]">Ctrl+K</kbd>
          <span>{{ t('sidebar.openCommandPaletteShortcut') }}</span>
        </div>
      </div>
    </div>

    <div v-else-if="filteredPlugins.length === 0 && plugins.length > 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('plugins.searchNoResult') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-content-muted">{{ t('plugins.emptyDesc') }}</p>
      <button @click="searchQuery = ''" class="mt-3 text-sm text-primary-600 hover:text-primary-700 dark:text-brand-primary">{{ t('common.clearSearch') }}</button>
    </div>

    <div v-else class="space-y-0">
      <div v-if="isBatchMode && selectedPluginCount > 0" class="bg-primary-50 dark:bg-surface-hover border border-primary-200 dark:border-surface-border rounded-[6px] p-2 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium text-primary-700 dark:text-content-secondary">{{ t('plugins.selectedCount', { count: selectedPluginCount }) }}</span>
          <button
            @click="selectAllPlugins"
            class="text-xs text-primary-600 dark:text-brand-primary hover:underline"
          >
            {{ t('plugins.batchActions.selectAll') }}
          </button>
          <button
            @click="clearPluginSelection"
            class="text-xs text-gray-500 dark:text-content-muted hover:underline"
          >
            {{ t('plugins.batchActions.deselectAll') }}
          </button>
        </div>
        <div class="flex gap-2">
          <button
            @click="batchRemovePlugins"
            class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-btn hover:bg-red-700 transition-colors"
          >
            {{ t('plugins.batchActions.batchDelete', { count: selectedPluginCount }) }}
          </button>
        </div>
      </div>

      <div class="space-y-0">
        <div
          v-for="plugin in displayedPlugins"
          :key="plugin.plugin_id"
          :class="[
            'px-3 py-2 border-b border-gray-200 dark:border-surface-border/40 hover:bg-gray-50 dark:hover:bg-white/[0.04] transition-colors duration-100',
            selectedPluginIds.has(plugin.plugin_id) ? 'bg-primary-50/50 dark:bg-primary-900/10' : ''
          ]"
          @contextmenu="showPluginContextMenu($event, plugin)"
          >
          <div class="flex items-start gap-2">
            <input
              type="checkbox"
              :checked="selectedPluginIds.has(plugin.plugin_id)"
              @click.stop="togglePluginSelection(plugin, $event)"
              class="checkbox-field mt-1"
            />
            <div class="w-8 h-8 rounded-[4px] overflow-hidden bg-primary-50 dark:bg-surface-hover flex items-center justify-center flex-shrink-0">
              <span class="text-primary-600 dark:text-brand-primary font-semibold text-xs">{{ plugin.name.charAt(0).toUpperCase() }}</span>
            </div>
            <div class="min-w-0 flex-1 cursor-pointer" @click="showPluginDetails(plugin)">
              <div class="flex items-center gap-2 flex-wrap">
                <h3 class="text-sm font-medium text-gray-900 dark:text-content-primary">
                  {{ plugin.name }}
                </h3>
                <span :class="['badge text-xs', plugin.compatibility === 'Godot4' ? 'bg-blue-100 text-blue-700 dark:bg-surface-hover dark:text-brand-primary' : plugin.compatibility === 'Godot3' ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : plugin.compatibility === 'Both' ? 'bg-purple-100 text-purple-700 dark:bg-surface-hover dark:text-content-secondary' : 'bg-gray-100 text-gray-500 dark:bg-surface-hover dark:text-content-muted']">
                  {{ plugin.compatibility === 'Godot4' ? '4.x' : plugin.compatibility === 'Godot3' ? '3.x' : plugin.compatibility === 'Both' ? '3/4' : '?' }}
                </span>
                <span class="badge badge-neutral text-xs">
                  {{ plugin.source.source_type === 'Local' ? t('plugins.source.local') : plugin.source.source_type === 'Git' ? t('plugins.source.git') : t('plugins.source.assetlibrary') }}
                </span>
                <span v-if="plugin.asset_type && plugin.asset_type !== 'Plugin'" :class="['badge text-xs', plugin.asset_type === 'AssetPack' ? 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400' : 'bg-teal-100 text-teal-700 dark:bg-teal-900/30 dark:text-teal-400']">
                  {{ plugin.asset_type === 'AssetPack' ? t('plugins.assetType.assetPack') : t('plugins.assetType.project') }}
                </span>
                <span v-if="pluginBindingCountMap.get(plugin.plugin_id)" class="badge bg-blue-100 text-blue-700 dark:bg-surface-hover dark:text-brand-primary text-xs">
                  {{ pluginBindingCountMap.get(plugin.plugin_id) }} {{ t('linker.projects') }}
                </span>
                <span class="text-xs text-gray-400 ml-auto flex-shrink-0">v{{ plugin.versions[0]?.version || '1.0.0' }}</span>
              </div>
              <div class="flex items-center gap-2 mt-0.5">
                <p class="text-xs text-gray-500 dark:text-content-secondary truncate" :title="plugin.description || t('plugins.noDescription')">
                  {{ plugin.description || t('plugins.noDescription') }}
                </p>
                <span v-if="plugin.author" class="text-xs text-gray-400 flex-shrink-0">{{ plugin.author }}</span>
              </div>
            </div>
          </div>
          <div class="flex items-center justify-between mt-1 pt-1 border-t border-gray-200 dark:border-surface-border/40">
            <div class="flex items-center gap-1">
              <button
                @click.stop="toggleFavorite(plugin)"
                :class="['p-1 rounded-[4px] transition-colors', plugin.is_favorite ? 'text-yellow-500 hover:text-yellow-600' : 'text-gray-400 dark:text-content-secondary hover:text-yellow-500']"
              >
                <svg class="w-4 h-4" :fill="plugin.is_favorite ? 'currentColor' : 'none'" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                </svg>
              </button>
              <span v-if="plugin.versions.length > 1" class="text-xs text-gray-400 dark:text-content-muted">{{ plugin.versions.length }} {{ t('plugins.versionCount') }}</span>
              <span v-if="plugin.versions[0]?.units.length > 1" class="text-xs text-gray-400 dark:text-content-muted">{{ plugin.versions[0].units.length }} {{ t('plugins.unitCount') }}</span>
            </div>
            <div class="flex items-center gap-2">
              <button
                v-if="plugin.source.source_type === 'Git'"
                @click.stop="updateGitPlugin(plugin.plugin_id)"
                class="text-primary-600 dark:text-brand-primary hover:text-primary-700 dark:hover:text-brand-primary p-1.5 rounded-[4px] hover:bg-primary-50 dark:hover:bg-surface-hover transition-colors"
                :title="t('plugins.contextMenu.updatePlugin')"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
              <button
                @click.stop="quickBindFromCard(plugin)"
                class="btn-primary text-sm flex items-center gap-1.5"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
                {{ t('plugins.bindToProject') }}
              </button>
              <button
                @click.stop="confirmRemovePlugin(plugin.plugin_id)"
                class="text-red-400 hover:text-red-600 p-1.5 rounded-[4px] hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
              >
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="hasMorePlugins" class="text-center py-3">
      <button
        @click="loadMorePlugins"
        class="px-3 py-1.5 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card text-gray-700 dark:text-content-secondary rounded-btn hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors text-sm"
      >
        {{ t('common.loadMore') }}
      </button>
    </div>
    </div>

  <Teleport to="body">
    <div v-if="showRemoteDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRemoteDialog = false; remoteUrl = ''; remoteGitRef = ''; gitRefs = []; gitRefDecisionMade = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.importFromRemote') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">
          {{ t('plugins.remoteImport.desc') }}
        </p>
        <input
          v-model="remoteUrl"
          type="text"
          :placeholder="t('plugins.remoteImport.placeholder')"
          @input="onRemoteUrlChange"
          class="w-full input-field text-sm"
        />
        <div v-if="isLoadingGitRefs" class="mt-2 text-xs text-gray-400 dark:text-content-muted">{{ t('plugins.remoteImport.loadingRefs') }}</div>
        <div v-if="gitRefs.length > 0" class="mt-2">
          <div class="text-xs text-gray-500 dark:text-content-secondary mb-1">{{ t('plugins.remoteImport.selectRef') }}</div>
          <div class="max-h-40 overflow-y-auto border border-gray-200/60 dark:border-surface-border/40 rounded-[6px]">
            <button
              @click="remoteGitRef = ''; gitRefDecisionMade = true"
              :class="['w-full text-left px-3 py-1.5 text-sm hover:bg-gray-50 dark:hover:bg-surface-layer flex items-center gap-2', !remoteGitRef && gitRefDecisionMade ? 'bg-primary-50 dark:bg-surface-hover text-primary-600 dark:text-brand-primary' : 'text-gray-700 dark:text-content-secondary']"
            >
              <span class="px-1.5 py-0.5 rounded-[4px] text-[10px] font-medium bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400">{{ t('plugins.remoteImport.useDefault') }}</span>
              <span>{{ t('plugins.remoteImport.defaultBranch') }}</span>
            </button>
            <button
              v-for="ref_item in gitRefs"
              :key="ref_item.name"
              @click="remoteGitRef = ref_item.name; gitRefDecisionMade = true"
              :class="['w-full text-left px-3 py-1.5 text-sm hover:bg-gray-50 dark:hover:bg-surface-layer flex items-center gap-2', remoteGitRef === ref_item.name ? 'bg-primary-50 dark:bg-surface-hover text-primary-600 dark:text-brand-primary' : 'text-gray-700 dark:text-content-secondary']"
            >
              <span :class="['px-1.5 py-0.5 rounded-[4px] text-[10px] font-medium', ref_item.ref_type === 'tag' ? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400' : 'bg-blue-100 dark:bg-surface-hover text-blue-700 dark:text-brand-primary']">{{ ref_item.ref_type === 'tag' ? 'tag' : 'branch' }}</span>
              <span>{{ ref_item.name }}</span>
            </button>
          </div>
        </div>
        <input
          v-model="remoteGitRef"
          type="text"
          :placeholder="t('plugins.remoteImport.refPlaceholder')"
          class="w-full input-field text-sm mt-3"
        />
        <div class="flex justify-end space-x-2 mt-4">
          <button
            @click="showRemoteDialog = false; remoteUrl = ''; remoteGitRef = ''; gitRefs = []; gitRefDecisionMade = false"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="importFromRemote"
            :disabled="isLoading || !remoteUrl || isLoadingGitRefs || (gitRefs.length > 0 && !gitRefDecisionMade && !remoteGitRef.trim())"
            class="btn-primary disabled:opacity-50"
          >
            {{ t('plugins.importFromProject.startImport') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showPluginDetail && selectedPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="closePluginDetail">
      <div class="dialog-container w-full max-w-lg max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex items-center justify-between mb-2">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">
            {{ selectedPlugin.name }}
          </h3>
          <div class="flex items-center gap-2">
            <button
              @click="quickBindFromCard(selectedPlugin)"
              class="btn-primary text-xs flex items-center gap-1"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>
              {{ t('plugins.bindToProject') }}
            </button>
            <button
              v-if="selectedPlugin.source.source_type === 'Git'"
              @click="updateGitPlugin(selectedPlugin.plugin_id); closePluginDetail()"
              class="px-3 py-1 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary text-xs rounded-btn hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors flex items-center gap-1"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
              {{ t('plugins.contextMenu.updatePlugin') }}
            </button>
            <button
              @click="openGlobalUpgradeDialog(selectedPlugin)"
              class="px-3 py-1 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary text-xs rounded-btn hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors flex items-center gap-1"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" /></svg>
              {{ t('batchOps.globalUpgrade') }}
            </button>
            <button @click="closePluginDetail" class="text-gray-500 dark:text-content-secondary hover:text-gray-700 dark:hover:text-content-primary text-sm">
              {{ t('common.close') }}
            </button>
          </div>
        </div>
        <div class="mb-2 flex items-center gap-2 flex-wrap text-sm text-gray-500 dark:text-content-secondary">
          <span>{{ t('plugins.author') }}: {{ selectedPlugin.author || t('plugins.unknownAuthor') }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ selectedPlugin.compatibility === 'Godot4' ? 'Godot 4' : selectedPlugin.compatibility === 'Godot3' ? 'Godot 3' : selectedPlugin.compatibility === 'Both' ? t('plugins.compat.both') : t('plugins.compat.unknown') }}</span>
          <span class="text-gray-300 dark:text-content-secondary">|</span>
          <span>{{ selectedPlugin.source.source_type === 'Local' ? t('plugins.source.local') : selectedPlugin.source.source_type === 'Git' ? t('plugins.source.git') : t('plugins.source.assetlibrary') }}</span>
          <span v-if="pluginStorageStats" class="text-gray-300 dark:text-content-secondary">|</span>
          <span v-if="pluginStorageStats">{{ pluginStorageStats.total_size_display }}</span>
        </div>

        <div class="flex-1 overflow-y-auto space-y-2">
          <div>
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('plugins.description') }}</h4>
            <p class="text-sm text-gray-600 dark:text-content-secondary whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3">
              {{ selectedPlugin.description || t('plugins.noDescription') }}
            </p>
          </div>

          <div>
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">
              {{ t('plugins.pluginDetail.versionList', { count: selectedPlugin.versions.length }) }}
            </h4>
            <div class="space-y-2 bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3">
              <div v-for="version in selectedPlugin.versions" :key="version.version_id"
                class="flex items-center justify-between py-1.5 border-b border-gray-200/60 dark:border-surface-border/40 last:border-0">
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
            <div class="space-y-1 bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3">
              <div v-for="binding in pluginBindings" :key="binding.project_id + binding.mount_path"
                class="flex items-center justify-between py-1">
                <div class="flex items-center gap-2 min-w-0">
                  <span v-if="binding.is_healthy === false" class="inline-flex items-center gap-1 text-xs text-red-500 flex-shrink-0">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                    </svg>
                    {{ t('plugins.bindDialog.broken') }}
                  </span>
                  <span v-else-if="binding.is_healthy === true" class="inline-flex items-center gap-1 text-xs text-green-500 flex-shrink-0">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
                    </svg>
                  </span>
                  <span v-else class="inline-flex items-center gap-1 text-xs text-gray-400 dark:text-content-muted flex-shrink-0">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01" />
                    </svg>
                    {{ t('plugins.bindDialog.unknown') }}
                  </span>
                  <span class="text-xs text-gray-900 dark:text-content-primary font-medium truncate">{{ bindingProjects.get(binding.project_id) || binding.project_id }}</span>
                  <span class="font-mono text-xs text-gray-500 dark:text-content-secondary flex-shrink-0">{{ binding.mount_path }}</span>
                </div>
                <div class="flex items-center gap-1 ml-2 flex-shrink-0">
                  <button
                    v-if="binding.is_healthy === false"
                    @click="repairBinding(binding.project_id, binding.plugin_id)"
                    class="text-xs text-primary-600 dark:text-brand-primary hover:underline"
                  >
                    {{ t('plugins.bindDialog.repair') }}
                  </button>
                  <button
                    @click="unbindFromDetail(binding)"
                    class="text-xs text-red-500 hover:text-red-700 hover:underline"
                  >
                    {{ t('linker.unbind') }}
                  </button>
                </div>
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
                class="text-xs text-primary-600 dark:text-brand-primary hover:underline disabled:opacity-50"
              >
                {{ isInstallingDeps ? t('plugins.depDialog.installing') : t('plugins.depDialog.installMissing', { count: missingDepPluginIds.length }) }}
              </button>
            </div>
            <div class="space-y-2 bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3">
              <div v-for="dep in pluginDependencies" :key="dep.plugin_id" class="flex items-center justify-between text-sm">
                <div class="text-gray-600 dark:text-content-secondary">
                  <span class="font-medium">{{ plugins.find(p => p.plugin_id === dep.plugin_id)?.name || dep.plugin_id }}</span>
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
            <p class="text-sm text-gray-600 dark:text-content-secondary bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3">
              {{ t(`plugins.pluginDetail.sourceTypes.${selectedPlugin.source.source_type}`) }}
              <span v-if="selectedPlugin.source.url" class="block text-xs mt-1 break-all font-mono">
                <a v-if="selectedPlugin.source.url.startsWith('http')" :href="selectedPlugin.source.url" target="_blank" rel="noopener" class="text-blue-500 hover:text-blue-600 dark:text-brand-primary dark:hover:text-brand-accent underline">{{ selectedPlugin.source.url }}</a>
                <span v-else>{{ selectedPlugin.source.url }}</span>
              </span>
            </p>
          </div>

          <div v-if="pluginStorageStats" class="grid grid-cols-3 gap-2">
            <div class="bg-gray-50 dark:bg-surface-layer rounded-[6px] p-2 text-center">
              <div class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.version_count }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">{{ t('plugins.pluginDetail.sections.version') }}</div>
            </div>
            <div class="bg-gray-50 dark:bg-surface-layer rounded-[6px] p-2 text-center">
              <div class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.binding_count }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">{{ t('plugins.pluginDetail.sections.mount') }}</div>
            </div>
            <div class="bg-gray-50 dark:bg-surface-layer rounded-[6px] p-2 text-center">
              <div class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ pluginStorageStats.total_size_display }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">{{ t('plugins.pluginDetail.sections.storage') }}</div>
            </div>
          </div>
        </div>

        <div class="flex justify-end mt-4 pt-3 border-t border-gray-200/60 dark:border-surface-border/40">
          <button
            @click="closePluginDetail"
            class="btn-secondary"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

    <AssetLibraryTab
      v-if="activeTab === 'assetLibrary'"
      :active-tab="activeTab"
      :load-plugins="loadPlugins"
      :show-post-import-guide="showPostImportGuide"
    />

  <Teleport to="body">
    <div v-if="showUpdatesDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showUpdatesDialog = false">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <div class="flex justify-between items-center mb-2">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('plugins.updateCheck.title') }}</h3>
          <button @click="showUpdatesDialog = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="space-y-2 max-h-80 overflow-y-auto">
          <div v-if="pluginUpdates.length === 0" class="text-center py-8 text-gray-500 dark:text-content-muted">
            {{ t('plugins.updateCheck.noPlugins') }}
          </div>
          <div v-for="update in pluginUpdates" :key="update.plugin_id" class="bg-gray-50 dark:bg-surface-hover rounded-[6px] p-3">
            <div class="flex items-center justify-between">
              <div>
                <span class="font-medium text-gray-900 dark:text-content-primary">{{ update.plugin_name || update.plugin_id }}</span>
                <div class="text-sm text-gray-500 dark:text-content-muted mt-1">
                  {{ t('plugins.updateCheck.versionInfo', { current: update.current_version, latest: update.latest_version }) }}
                </div>
                <div v-if="update.affected_projects && update.affected_projects.length > 0" class="text-xs text-blue-600 dark:text-brand-primary mt-1">
                  {{ t('plugins.updateCheck.affectedProjects', { count: update.affected_projects.length }) }}: {{ update.affected_projects.join(', ') }}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button
                  v-if="update.update_available"
                  @click="updateGitPlugin(update.plugin_id)"
                  :disabled="isBatchUpdating"
                  class="btn-primary text-xs disabled:opacity-50"
                >
                  {{ t('plugins.updateCheck.update') }}
                </button>
                <span v-if="update.update_available" class="px-2 py-1 rounded-[4px] text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  {{ t('plugins.updateCheck.hasUpdate') }}
                </span>
                <span v-else class="px-2 py-1 rounded-[4px] text-xs font-medium bg-gray-100 text-gray-600 dark:bg-surface-layer dark:text-content-muted">
                  {{ t('plugins.updateCheck.upToDate') }}
                </span>
              </div>
            </div>
            <div v-if="update.release_notes" class="mt-2 pt-2 border-t border-gray-200/60 dark:border-surface-border/40">
              <button
                @click="expandedReleaseNotes.has(update.plugin_id) ? expandedReleaseNotes.delete(update.plugin_id) : expandedReleaseNotes.add(update.plugin_id)"
                class="text-xs text-primary-600 dark:text-brand-primary hover:underline"
              >
                {{ expandedReleaseNotes.has(update.plugin_id) ? t('plugins.updateCheck.hideNotes') : t('plugins.updateCheck.showNotes') }}
              </button>
              <div v-if="expandedReleaseNotes.has(update.plugin_id)" class="mt-2 text-xs text-gray-600 dark:text-content-secondary whitespace-pre-wrap max-h-32 overflow-y-auto bg-white dark:bg-surface-card rounded-[6px] p-2">
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
            class="btn-primary disabled:opacity-50 text-sm"
          >
            {{ isBatchUpdating ? t('plugins.updateCheck.updating') : t('plugins.updateCheck.updateAll', { count: updatablePluginIds.length }) }}
          </button>
          <div v-else></div>
          <button
            @click="showUpdatesDialog = false"
            class="btn-secondary"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showScanPreviewDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showScanPreviewDialog = false">
      <div class="dialog-container w-full max-w-lg max-h-[80vh] flex flex-col" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.importFromProject.scanTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">{{ t('plugins.importFromProject.scanDesc', { count: scannedPlugins.length }) }}</p>
        <div class="flex-1 overflow-y-auto space-y-2 mb-2">
          <div v-for="(plugin, idx) in scannedPlugins" :key="idx" class="bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3 flex items-center gap-2">
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ plugin.plugin_name }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary truncate">{{ plugin.project_name }} · {{ plugin.path }}</div>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showScanPreviewDialog = false" class="btn-secondary">{{ t('common.cancel') }}</button>
          <button @click="startImportFromPreview" class="btn-primary">{{ t('plugins.importFromProject.continueImport') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showImportModeDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showImportModeDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.importFromProject.title') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">{{ t('plugins.importFromProject.modeSelect') }}</p>
        <div class="space-y-2 mb-3">
          <label class="flex items-start gap-2 p-3 rounded-[6px] border cursor-pointer transition-colors"
            :class="importMode === 'copy' ? 'border-primary-500 bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40 hover:bg-gray-50 dark:hover:bg-surface-layer'">
            <input type="radio" v-model="importMode" value="copy" class="mt-1" />
            <div>
              <div class="font-medium text-gray-900 dark:text-content-primary text-sm">{{ t('plugins.importModes.copy.label') }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">{{ t('plugins.importModes.copy.desc') }}</div>
            </div>
          </label>
          <label class="flex items-start gap-2 p-3 rounded-[6px] border cursor-pointer transition-colors"
            :class="importMode === 'move' ? 'border-primary-500 bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40 hover:bg-gray-50 dark:hover:bg-surface-layer'">
            <input type="radio" v-model="importMode" value="move" class="mt-1" />
            <div>
              <div class="font-medium text-gray-900 dark:text-content-primary text-sm">{{ t('plugins.importModes.move.label') }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">{{ t('plugins.importModes.move.desc') }}</div>
            </div>
          </label>
          <label class="flex items-start gap-2 p-3 rounded-[6px] border cursor-pointer transition-colors"
            :class="importMode === 'reference' ? 'border-primary-500 bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40 hover:bg-gray-50 dark:hover:bg-surface-layer'">
            <input type="radio" v-model="importMode" value="reference" class="mt-1" />
            <div>
              <div class="font-medium text-gray-900 dark:text-content-primary text-sm">{{ t('plugins.importModes.reference.label') }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">{{ t('plugins.importModes.reference.desc') }}</div>
            </div>
          </label>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showImportModeDialog = false" class="btn-secondary">{{ t('plugins.importFromProject.cancel') }}</button>
          <button @click="doImportFromProjects" class="btn-primary">{{ t('plugins.importFromProject.startImport') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

    <div v-if="activeTab === 'repository' && totalStorageStats" class="border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-content-secondary">
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
        <div class="flex items-center gap-2">
          <button
            v-if="totalStorageStats.duplicate_hash_count > 0"
            @click="checkAndShowDuplicates"
            class="px-3 py-1 text-xs border border-yellow-300 dark:border-yellow-700 text-yellow-600 dark:text-yellow-400 rounded-btn hover:bg-yellow-50 dark:hover:bg-yellow-900/20"
          >
            {{ t('plugins.storageStats.viewDuplicates') }}
          </button>
          <button
            @click="cleanupOrphaned"
            :class="[
              'px-3 py-1 text-xs rounded-btn transition-colors',
              totalStorageStats.orphaned_size_bytes > 0
                ? 'border border-orange-300 dark:border-orange-700 text-orange-600 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/20'
                : 'border border-gray-200/60 dark:border-surface-border/40 text-gray-400 dark:text-content-muted cursor-default'
            ]"
            :disabled="totalStorageStats.orphaned_size_bytes === 0"
          >
            {{ t('plugins.storageStats.cleanup') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'bindings'" class="space-y-2">
      <div class="flex flex-wrap gap-2 items-center">
        <button
          @click="showGraphView = !showGraphView"
          class="px-3 py-1.5 border border-gray-300 dark:border-surface-border rounded-btn bg-white dark:bg-surface-card text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-layer"
        >
          {{ showGraphView ? t('linker.listView') : t('linker.graphView') }}
        </button>
        <button
          v-if="selectedLinkProjectCount > 1"
          @click="batchApplyChanges"
          class="px-3 py-1.5 border border-gray-300 dark:border-surface-border rounded-btn bg-white dark:bg-surface-card text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-layer"
        >
          {{ t('linker.batchApplyTitle') }}
        </button>
        <button
          v-if="selectedLinkId"
          @click="exportHarborConfig"
          :disabled="isExportingConfig"
          class="px-3 py-1.5 border border-gray-300 dark:border-surface-border rounded-btn bg-white dark:bg-surface-card text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-layer disabled:opacity-50"
        >
          {{ isExportingConfig ? t('linker.exporting') : t('linker.exportConfig') }}
        </button>
        <button
          v-if="selectedLinkId"
          @click="syncHarborConfig"
          :disabled="isSyncingConfig"
          class="px-3 py-1.5 border border-gray-300 dark:border-surface-border rounded-btn bg-white dark:bg-surface-card text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-layer disabled:opacity-50"
        >
          {{ isSyncingConfig ? t('linker.syncing') : t('linker.syncConfig') }}
        </button>
        <div v-if="mountStrategyDisplay" class="flex items-center gap-2 text-xs text-gray-500 dark:text-content-secondary">
          <span class="px-2 py-1 bg-gray-100 dark:bg-surface-layer rounded-[4px]">
            {{ t('plugins.mountStrategyLabel') }}: {{ mountStrategyDisplay }}
          </span>
          <span v-if="mountStrategyDisplay === 'Symlink'" class="hidden sm:inline">{{ t('settings.symlinkDesc') }}</span>
          <span v-else-if="mountStrategyDisplay === 'Junction'" class="hidden sm:inline">{{ t('settings.junctionDesc') }}</span>
          <span v-else-if="mountStrategyDisplay === 'Copy'" class="hidden sm:inline">{{ t('settings.copyDesc') }}</span>
          <span class="hidden sm:inline text-gray-400 dark:text-content-muted">{{ t('plugins.mountStrategyChangeHint') }}</span>
        </div>
      </div>

      <div v-if="!showGraphView" class="grid grid-cols-12 gap-2">
        <div class="col-span-3 border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-0">
          <div class="p-3 border-b border-gray-200/60 dark:border-surface-border/40">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('linker.projectList') }}</h3>
          </div>
          <div class="max-h-[calc(100vh-280px)] overflow-y-auto">
            <div v-if="linkerProjects.length === 0" class="p-3 text-center">
              <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('linker.emptyProject') }}</h3>
              <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">{{ t('linker.emptyProjectDesc') }}</p>
            </div>
            <div
              v-for="project in linkerProjects"
              :key="project.project_id"
              @click="selectLinkProject(project, $event)"
              :class="['px-3 py-2 cursor-pointer border-b border-gray-200 dark:border-surface-border/40 last:border-0 transition-colors group', selectedLinkProjectIds.has(project.project_id) ? 'bg-primary-50 dark:bg-surface-hover' : 'hover:bg-gray-50 dark:hover:bg-surface-layer']"
            >
              <div class="flex items-center gap-1.5">
                <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex-1 min-w-0">{{ project.name }}</div>
                <span
                  v-if="harborConfigStatus.get(project.project_id)"
                  class="shrink-0 text-[10px] px-1.5 py-0.5 rounded-[4px] bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 font-medium"
                  :title="t('linker.configExists')"
                >.harbor.yml</span>
              </div>
              <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-content-secondary">
                <span>{{ project.godot_version }}</span>
                <span v-if="linkerProjectBindingCounts.get(project.project_id)" class="text-blue-500 dark:text-brand-primary">{{ linkerProjectBindingCounts.get(project.project_id) }} {{ t('linker.bindingCountShort') }}</span>
              </div>
              <div v-if="linkerProjectBindingNames.get(project.project_id)?.length" class="mt-1 flex flex-wrap gap-1">
                <span
                  v-for="name in linkerProjectBindingNames.get(project.project_id)"
                  :key="name"
                  class="inline-block px-1.5 py-0.5 text-[10px] bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-muted rounded-[4px]"
                >{{ name }}</span>
                <span v-if="(linkerProjectBindingCounts.get(project.project_id) || 0) > 3" class="inline-block px-1.5 py-0.5 text-[10px] text-gray-400 dark:text-content-muted">+{{ (linkerProjectBindingCounts.get(project.project_id) || 0) - 3 }}</span>
              </div>
              <div class="mt-1.5 flex gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  @click.stop="selectedLinkId = project.project_id; exportHarborConfig()"
                  class="text-[10px] px-1.5 py-0.5 rounded-[4px] bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-muted hover:bg-primary-100 dark:hover:bg-surface-hover hover:text-primary-600 dark:hover:text-brand-primary"
                >{{ t('linker.exportConfig') }}</button>
                <button
                  @click.stop="selectedLinkId = project.project_id; syncHarborConfig()"
                  class="text-[10px] px-1.5 py-0.5 rounded-[4px] bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-muted hover:bg-primary-100 dark:hover:bg-surface-hover hover:text-primary-600 dark:hover:text-brand-primary"
                >{{ t('linker.syncConfig') }}</button>
              </div>
            </div>
          </div>
        </div>

        <div class="col-span-5 border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-0">
          <div class="p-3 border-b border-gray-200/60 dark:border-surface-border/40">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('linker.availablePlugins') }}</h3>
              <button
                v-if="selectedLinkProjectCount > 0 && selectedLinkPluginCount > 0"
                @click="batchBindPlugins"
                class="btn-primary text-xs"
              >
                {{ t('linker.batchBind', { count: selectedLinkPluginCount }) }}
              </button>
            </div>
            <input
              v-model="linkerSearchQuery"
              type="text"
              :placeholder="t('linker.searchPlugins')"
              class="w-full input-field text-xs"
            />
          </div>
          <div class="max-h-[calc(100vh-280px)] overflow-y-auto">
            <div v-if="selectedLinkProjectCount === 0" class="p-3 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ t('linker.selectProject') }}
            </div>
            <div v-else-if="filteredUnboundPlugins.length === 0" class="p-3 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ linkerSearchQuery ? t('linker.noSearchResults') : t('linker.allPluginsBound') }}
            </div>
            <div
              v-for="plugin in filteredUnboundPlugins"
              :key="plugin.plugin_id"
              @click="toggleLinkPluginSelection(plugin.plugin_id, $event)"
              :class="['px-3 py-2 cursor-pointer border-b border-gray-200 dark:border-surface-border/40 last:border-0 transition-colors', selectedLinkPluginIds.has(plugin.plugin_id) ? 'bg-primary-50 dark:bg-surface-hover' : 'hover:bg-gray-50 dark:hover:bg-surface-layer']"
            >
              <div class="flex items-center justify-between">
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1">
                    {{ plugin.name }}
                    <span v-if="selectedLinkProjectIds.size > 0 && Array.from(selectedLinkProjectIds).some(pid => { const proj = linkerProjects.find(p => p.project_id === pid); return proj && isCompatWarning(plugin, proj); })" class="text-xs text-orange-500 dark:text-orange-400" :title="t('plugins.bindDialog.compatWarning')">⚠</span>
                  </div>
                  <div class="text-xs text-gray-500 dark:text-content-secondary">v{{ plugin.versions[0]?.version || '1.0.0' }} · {{ plugin.author || t('linker.unknown') }}</div>
                </div>
                <button @click.stop="bindPluginToProject(plugin)" class="btn-primary text-xs ml-2 flex-shrink-0">
                  {{ t('linker.bind') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="col-span-4 border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-0">
          <div class="p-3 border-b border-gray-200/60 dark:border-surface-border/40 flex items-center justify-between">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('linker.boundPlugins') }}</h3>
            <button
              v-if="boundPluginNames.some(b => selectedLinkPluginIds.has(b.plugin_id))"
              @click="batchUnbindPlugins"
              class="px-2 py-1 bg-red-600 text-white text-xs rounded-btn hover:bg-red-700"
            >
              {{ t('linker.batchUnbind', { count: linkerBindings.filter(b => selectedLinkPluginIds.has(b.plugin_id)).length }) }}
            </button>
          </div>
          <div class="max-h-[calc(100vh-280px)] overflow-y-auto">
            <div v-if="!selectedLinkId" class="p-3 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ t('linker.selectProject') }}
            </div>
            <div v-else-if="boundPluginNames.length === 0" class="p-3 text-center text-sm text-gray-500 dark:text-content-secondary">
              {{ t('linker.noBindings') }}
            </div>
            <div
              v-for="item in boundPluginNames"
              :key="`${item.project_id}-${item.plugin_id}`"
              :class="['px-3 py-2 border-b border-gray-200 dark:border-surface-border/40 last:border-0', item.is_healthy === false ? 'bg-red-50 dark:bg-red-900/10' : '']"
            >
              <div class="flex items-center justify-between">
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1.5">
                    {{ item.plugin?.name || t('linker.unknownPlugin') }}
                    <span v-if="item.is_healthy === false" class="w-2 h-2 rounded-full bg-red-500 flex-shrink-0" :title="t('plugins.bindDialog.unhealthy')"></span>
                  </div>
                  <div class="text-xs text-gray-500 dark:text-content-secondary flex items-center gap-2">
                    <span v-if="selectedLinkProjectIds.size > 1" class="text-blue-500 dark:text-brand-primary">{{ linkerProjects.find(p => p.project_id === item.project_id)?.name || item.project_id }}</span>
                    <span>{{ item.mount_path }}</span>
                    <span v-if="getBindingVersion(item)" class="text-blue-500 dark:text-brand-primary">v{{ getBindingVersion(item) }}</span>
                  </div>
                </div>
                <button @click="unbindPluginFromProject(item)" class="px-2 py-1 text-red-600 dark:text-red-400 text-xs hover:bg-red-50 dark:hover:bg-red-900/20 rounded-btn ml-2 flex-shrink-0">
                  {{ t('linker.unbind') }}
                </button>
                <button @click="openVersionSwitch(item)" class="px-2 py-1 text-primary-600 dark:text-brand-primary text-xs hover:bg-primary-50 dark:hover:bg-surface-hover rounded-btn ml-1 flex-shrink-0" :title="t('linker.switchVersion')">
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-3">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('linker.bindingGraph') }}</h3>
        <div v-if="graphLinks.length === 0" class="text-center py-8 text-sm text-gray-500 dark:text-content-muted">
          {{ t('linker.noBindings') }}
        </div>
        <svg v-else width="100%" :height="graphSvgHeight" :viewBox="`0 0 800 ${graphSvgHeight}`" class="border border-gray-200/60 dark:border-surface-border/40 rounded-[6px]">
          <text x="120" y="24" text-anchor="middle" class="fill-gray-500 dark:fill-gray-400" font-size="11">{{ t('linker.projects') }}</text>
          <text x="680" y="24" text-anchor="middle" class="fill-gray-500 dark:fill-gray-400" font-size="11">{{ t('linker.plugins') }}</text>
          <g v-for="node in graphNodes.filter(n => n.type === 'project')" :key="'p-' + node.id">
            <rect :x="10" :y="node.y - 14" width="220" height="28" rx="6" class="fill-blue-100 dark:fill-surface-hover stroke-blue-300 dark:stroke-surface-border" stroke-width="1"/>
            <text :x="120" :y="node.y + 4" text-anchor="middle" class="fill-gray-700 dark:fill-gray-300" font-size="11">{{ node.name }}</text>
          </g>
          <g v-for="node in graphNodes.filter(n => n.type === 'plugin')" :key="'pl-' + node.id">
            <rect :x="570" :y="node.y - 14" width="220" height="28" rx="6" class="fill-green-100 dark:fill-green-900/30 stroke-green-300 dark:stroke-green-700" stroke-width="1"/>
            <text :x="680" :y="node.y + 4" text-anchor="middle" class="fill-gray-700 dark:fill-gray-300" font-size="11">{{ node.name }}</text>
          </g>
          <line v-for="link in graphLinks" :key="link.projectId + '-' + link.pluginId" :x1="230" :y1="graphNodes.find(n => n.id === link.projectId)?.y" :x2="570" :y2="graphNodes.find(n => n.id === link.pluginId)?.y" class="stroke-gray-400 dark:stroke-gray-500" stroke-width="1" stroke-dasharray="4"/>
        </svg>
      </div>

    </div>
    </div>
  </div>

  <Teleport to="body">
    <div v-if="showDeletePluginConfirm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showDeletePluginConfirm = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.deleteConfirm.single') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-3">
          {{ t('plugins.deleteConfirm.singleDesc') }}
        </p>
        <div v-if="deletePluginBindings.length > 0" class="mb-2 p-3 bg-red-50 dark:bg-red-900/20 border-2 border-red-300 dark:border-red-700 rounded-[6px]">
          <div class="flex items-center gap-2 mb-2">
            <svg class="w-5 h-5 text-red-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
            <p class="text-sm font-bold text-red-700 dark:text-red-400">
              {{ t('plugins.deleteConfirm.bindingWarning', { count: new Set(deletePluginBindings.map(b => b.project_id)).size, name: deletePluginName }) }}
            </p>
          </div>
          <div class="space-y-2 max-h-40 overflow-y-auto">
            <div v-for="projectId in [...new Set(deletePluginBindings.map(b => b.project_id))]" :key="projectId" class="text-xs"><div class="font-medium text-red-700 dark:text-red-400">{{ deletePluginProjects.get(projectId) || projectId }}</div><div v-for="binding in deletePluginBindings.filter(b => b.project_id === projectId)" :key="binding.mount_path" class="text-red-600 dark:text-red-400 pl-3">{{ binding.mount_path }}</div></div>
          </div>
          <p class="text-xs text-red-500 dark:text-red-400 mt-2 font-medium">
            {{ t('plugins.deleteConfirm.bindingWarningDesc') }}
          </p>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showDeletePluginConfirm = false" class="btn-secondary">{{ t('common.cancel') }}</button>
          <button @click="onRemovePluginConfirm(); showDeletePluginConfirm = false" class="px-3 py-1.5 bg-red-600 text-white rounded-btn hover:bg-red-700 text-sm">{{ t('plugins.deleteConfirm.singleConfirm') }}</button>
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

    <ConfirmDialog
      v-model="showVersionDeleteConfirm"
      :title="t('plugins.versionDeleted')"
      :description="versionDeleteWarning"
      :confirm-text="t('common.confirmDelete')"
      confirm-color="red"
      @confirm="onVersionDeleteConfirm"
    />

    <ConfirmDialog
      v-model="showDeleteConfigConfirm"
      :title="t('linker.deleteConfig')"
      :description="t('linker.deleteConfigConfirm')"
      :confirm-text="t('common.confirmDelete')"
      confirm-color="red"
      @confirm="onConfirmDeleteConfig"
    />

    <ConfirmDialog
      v-model="showCleanupConfirm"
      :title="t('plugins.cleanupOrphaned.title')"
      :description="t('plugins.cleanupOrphaned.confirmDesc')"
      :confirm-text="t('plugins.cleanupOrphaned.title')"
      confirm-color="orange"
      @confirm="onConfirmCleanup"
    />

  </Teleport>

  <Teleport to="body">
    <div v-if="showDuplicateConfirm && duplicateCheckResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showDuplicateConfirm = false; duplicateCheckResult = null; pendingImportAction = null">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.duplicate.title') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">
          {{ t('plugins.duplicate.desc', { name: duplicateCheckResult.duplicate_plugin_name || duplicateCheckResult.duplicate_plugin_id || '' }) }}
        </p>
        <div class="flex justify-end gap-2">
          <button @click="showDuplicateConfirm = false; duplicateCheckResult = null; pendingImportAction = null" class="btn-secondary">{{ t('plugins.duplicate.cancel') }}</button>
          <button
            @click="(() => { const dupId = duplicateCheckResult?.duplicate_plugin_id; showDuplicateConfirm = false; duplicateCheckResult = null; if (dupId) { removePluginAndReimport(dupId); } })()"
            class="px-3 py-1.5 bg-orange-600 text-white rounded-btn hover:bg-orange-700 text-sm"
          >
            {{ t('plugins.duplicate.replaceExisting') }}
          </button>
          <button
            @click="showDuplicateConfirm = false; duplicateCheckResult = null; if (pendingImportAction) { pendingImportAction(); pendingImportAction = null; }"
            class="btn-primary text-sm"
          >
            {{ t('plugins.duplicate.importAnyway') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showLinkerVersionSelect && versionSelectPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerVersionSelect = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.versionSelectTitle', { name: versionSelectPlugin.name }) }}</h3>
        <div class="space-y-2">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('linker.selectVersion') }}</label>
            <select v-model="selectedVersionIdx" class="w-full select-field">
              <option v-for="(v, i) in versionSelectPlugin.versions" :key="v.version_id" :value="i">{{ v.version }}</option>
            </select>
          </div>
          <div v-if="versionSelectPlugin.versions[selectedVersionIdx]?.units.length > 1">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('linker.selectUnit') }}</label>
            <select v-model="selectedUnitIdx" class="w-full select-field">
              <option v-for="(u, i) in versionSelectPlugin.versions[selectedVersionIdx]?.units" :key="u.unit_id" :value="i">{{ u.name }}</option>
            </select>
          </div>
          <p class="text-xs text-gray-500 dark:text-content-secondary">
            {{ t('linker.mountPath') }}: {{ getMountPath(versionSelectPlugin.versions[selectedVersionIdx]?.units[selectedUnitIdx], versionSelectPlugin) }}
          </p>
        </div>
        <div class="flex justify-end gap-2 mt-4">
          <button @click="showLinkerVersionSelect = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmVersionSelect" class="btn-primary">{{ t('linker.confirmBind') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showLinkerApplyResult && linkerApplyResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerApplyResult = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ linkerApplyResult.errors.length === 0 ? t('linker.applySuccess') : t('linker.applyWithErrors') }}</h3>
        <div v-if="linkerApplyResult.created.length > 0" class="mb-3">
          <p class="text-sm font-medium text-green-600 dark:text-green-400 mb-1">{{ t('linker.created') }}:</p>
          <ul class="text-xs text-gray-600 dark:text-content-muted list-disc list-inside">
            <li v-for="item in linkerApplyResult.created" :key="item">{{ item }}</li>
          </ul>
        </div>
        <div v-if="linkerApplyResult.removed.length > 0" class="mb-3">
          <p class="text-sm font-medium text-yellow-600 dark:text-yellow-400 mb-1">{{ t('linker.removed') }}:</p>
          <ul class="text-xs text-gray-600 dark:text-content-muted list-disc list-inside">
            <li v-for="item in linkerApplyResult.removed" :key="item">{{ item }}</li>
          </ul>
        </div>
        <div v-if="linkerApplyResult.errors.length > 0" class="mb-3">
          <p class="text-sm font-medium text-red-600 dark:text-red-400 mb-1">{{ t('linker.errors') }}:</p>
          <ul class="text-xs text-gray-600 dark:text-content-muted list-disc list-inside">
            <li v-for="item in linkerApplyResult.errors" :key="item">{{ item }}</li>
          </ul>
        </div>
        <div v-if="linkerApplyResult.errors.length === 0 && !autoApplyEnabled" class="mb-3 p-3 bg-blue-50 dark:bg-surface-hover rounded-[6px]">
          <p class="text-xs text-blue-700 dark:text-content-secondary mb-2">{{ t('plugins.autoApplyPrompt') }}</p>
          <button @click="goToAutoApplySettings" class="text-xs font-medium text-blue-600 dark:text-brand-primary hover:underline">{{ t('plugins.autoApplyPromptAction') }} &rarr;</button>
        </div>
        <div v-if="linkerApplyResult.removed.length > 0 || linkerApplyResult.created.length > 0" class="mb-3">
          <button @click="loadAddonBackups" class="text-xs font-medium text-orange-600 dark:text-orange-400 hover:underline">{{ t('plugins.rollbackAddons') }}</button>
        </div>
        <div class="flex justify-end">
          <button @click="showLinkerApplyResult = false" class="btn-primary">{{ t('linker.close') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showRollbackDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRollbackDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.rollbackAddons') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">{{ t('plugins.rollbackAddonsDesc') }}</p>
        <div v-if="addonBackups.length === 0" class="text-sm text-gray-400 dark:text-content-muted py-3 text-center">{{ t('plugins.noBackups') }}</div>
        <div v-else class="max-h-60 overflow-y-auto space-y-2">
          <div v-for="backup in addonBackups" :key="backup.file_name" class="flex items-center justify-between p-3 bg-gray-50 dark:bg-surface-hover/50 rounded-[6px]">
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ backup.created_at }}</p>
              <p class="text-xs text-gray-500 dark:text-content-muted">{{ (backup.file_size / 1024).toFixed(1) }} KB</p>
            </div>
            <button @click="doRestoreAddonBackup(backup.file_path)" :disabled="isRestoringAddon" class="px-3 py-1 text-xs bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-300 rounded-btn hover:bg-orange-200 dark:hover:bg-orange-900/50 disabled:opacity-50">{{ t('plugins.rollbackAddons') }}</button>
          </div>
        </div>
        <div class="flex justify-end mt-4">
          <button @click="showRollbackDialog = false" class="btn-secondary">{{ t('common.close') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showLinkerBatchBindDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchBindDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.batchBindTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">
          {{ t('linker.batchBindDesc', { projectCount: selectedLinkProjectCount, pluginCount: selectedLinkPluginCount }) }}
        </p>
        <div v-if="batchProgress" class="mb-2">
          <div class="flex items-center justify-between text-xs text-gray-600 dark:text-content-secondary mb-1">
            <span>{{ batchProgress.message }}</span>
            <span>{{ Math.round((batchProgress.current / batchProgress.total) * 100) }}%</span>
          </div>
          <div class="w-full bg-gray-200 dark:bg-surface-hover rounded-full h-2">
            <div class="bg-primary-600 h-2 rounded-full transition-all duration-300" :style="{ width: `${(batchProgress.current / batchProgress.total) * 100}%` }"></div>
          </div>
        </div>
        <div class="mb-2">
          <p class="text-xs font-medium text-gray-500 dark:text-content-muted mb-1">{{ t('linker.targetProjects') }}：</p>
          <div class="flex flex-wrap gap-1">
            <span v-for="id in selectedLinkProjectIds" :key="id" class="text-xs bg-blue-100 dark:bg-surface-hover text-blue-700 dark:text-content-secondary px-2 py-0.5 rounded-[4px]">
              {{ linkerProjects.find(p => p.project_id === id)?.name || id }}
            </span>
          </div>
        </div>
        <div class="mb-2">
          <p class="text-xs font-medium text-gray-500 dark:text-content-muted mb-1">{{ t('linker.bindPlugins') }}：</p>
          <div class="flex flex-wrap gap-1">
            <span v-for="id in selectedLinkPluginIds" :key="id" class="text-xs bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 px-2 py-0.5 rounded-[4px]">
              {{ plugins.find(p => p.plugin_id === id)?.name || t('linker.unknownPlugin') }}
            </span>
          </div>
        </div>
        <div v-if="batchFailedItems.length > 0" class="mb-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-[6px]">
          <p class="text-xs font-medium text-red-600 dark:text-red-400 mb-2">{{ batchFailedItems.length }} {{ t('plugins.retryFailed') }}</p>
          <button @click="retryBatchFailed" class="px-3 py-1 bg-red-600 text-white text-xs rounded-btn hover:bg-red-700">{{ t('plugins.retryFailed') }}</button>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showLinkerBatchBindDialog = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmBatchBind" :disabled="isLinkerBatchBinding" class="btn-primary disabled:opacity-50">
            {{ isLinkerBatchBinding ? t('linker.batchBinding') : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showLinkerBatchUnbindDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchUnbindDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.batchUnbindTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">
          {{ t('linker.batchUnbindDesc', { projectName: linkerProjects.find(p => p.project_id === selectedLinkId)?.name, count: linkerBindings.filter(b => selectedLinkPluginIds.has(b.plugin_id)).length }) }}
        </p>
        <div class="mb-2">
          <div class="flex flex-wrap gap-1">
            <span v-for="id in selectedLinkPluginIds" :key="id" class="text-xs bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 px-2 py-0.5 rounded-[4px]">
              {{ plugins.find(p => p.plugin_id === id)?.name || t('linker.unknownPlugin') }}
            </span>
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showLinkerBatchUnbindDialog = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmBatchUnbind" :disabled="isLinkerBatchUnbinding" class="px-3 py-1.5 bg-red-600 text-white rounded-btn hover:bg-red-700 text-sm disabled:opacity-50">
            {{ isLinkerBatchUnbinding ? t('linker.batchUnbinding') : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showLinkerUnbindConfirm && pendingUnbindBinding" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerUnbindConfirm = false; pendingUnbindBinding = null">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.unbindConfirm') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">
          {{ t('linker.unbindConfirmDesc', { name: plugins.find(p => p.plugin_id === pendingUnbindBinding?.plugin_id)?.name || pendingUnbindBinding?.plugin_id || '' }) }}
        </p>
        <div class="flex justify-end gap-2">
          <button @click="showLinkerUnbindConfirm = false; pendingUnbindBinding = null" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmUnbindPlugin" class="px-3 py-1.5 bg-red-600 text-white rounded-btn hover:bg-red-700 text-sm">
            {{ t('linker.unbind') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showBatchVersionSelectDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBatchVersionSelectDialog = false; batchVersionSelectMap = new Map()">
      <div class="dialog-container w-full max-w-lg max-h-[80vh] flex flex-col" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.batchVersionSelectTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">{{ t('linker.batchVersionSelectDesc') }}</p>
        <div class="flex-1 overflow-y-auto space-y-2">
          <div v-for="pluginId in batchVersionSelectPluginIds" :key="pluginId" class="bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3">
            <h4 class="text-sm font-medium text-gray-900 dark:text-content-primary mb-2">{{ plugins.find(p => p.plugin_id === pluginId)?.name || pluginId }}</h4>
            <div class="flex gap-2">
              <select
                :value="batchVersionSelectMap.get(pluginId)?.versionIdx ?? 0"
                @change="(() => { const m = new Map(batchVersionSelectMap); m.set(pluginId, { versionIdx: Number(($event.target as HTMLSelectElement).value), unitIdx: 0 }); batchVersionSelectMap = m; })"
                class="flex-1 select-field text-xs"
              >
                <option v-for="(v, i) in plugins.find(p => p.plugin_id === pluginId)?.versions" :key="v.version_id" :value="i">v{{ v.version }}</option>
              </select>
              <select
                v-if="(plugins.find(p => p.plugin_id === pluginId)?.versions[batchVersionSelectMap.get(pluginId)?.versionIdx ?? 0]?.units?.length ?? 0) > 1"
                :value="batchVersionSelectMap.get(pluginId)?.unitIdx ?? 0"
                @change="(() => { const m = new Map(batchVersionSelectMap); const cur = m.get(pluginId) || { versionIdx: 0, unitIdx: 0 }; m.set(pluginId, { ...cur, unitIdx: Number(($event.target as HTMLSelectElement).value) }); batchVersionSelectMap = m; })"
                class="flex-1 select-field text-xs"
              >
                <option v-for="(u, i) in plugins.find(p => p.plugin_id === pluginId)?.versions[batchVersionSelectMap.get(pluginId)?.versionIdx ?? 0]?.units" :key="u.unit_id" :value="i">{{ u.name }}</option>
              </select>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-4 pt-3 border-t border-gray-200/60 dark:border-surface-border/40">
          <button @click="showBatchVersionSelectDialog = false; batchVersionSelectMap = new Map()" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmBatchVersionSelect" class="btn-primary">{{ t('linker.confirmBind') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showLinkerBatchApplyDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchApplyDialog = false">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.batchApplyTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">{{ t('linker.batchApplyDesc') }}</p>
        <div class="space-y-2 mb-2 max-h-48 overflow-y-auto">
          <div v-for="id in selectedLinkProjectIds" :key="id" class="flex items-center justify-between text-sm">
            <span class="text-gray-900 dark:text-content-primary">{{ linkerProjects.find(p => p.project_id === id)?.name || id }}</span>
            <span class="text-xs text-gray-500 dark:text-content-secondary">{{ t('linker.bindingCountUnit', { count: linkerBindings.filter(b => b.project_id === id).length }) }}</span>
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showLinkerBatchApplyDialog = false" class="btn-secondary">{{ t('linker.cancel') }}</button>
          <button @click="confirmBatchApply" :disabled="isLinkerBatchApplying" class="btn-primary disabled:opacity-50">
            {{ isLinkerBatchApplying ? t('linker.batchApplying') : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showLinkerBatchApplyResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLinkerBatchApplyResult = false">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.batchApplyResultTitle') }}</h3>
        <div class="space-y-2 max-h-64 overflow-y-auto">
          <div v-for="result in batchApplyResults" :key="result.project_id" class="p-3 rounded-[6px] border" :class="result.success ? 'border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/10' : 'border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/10'">
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
  </Teleport>

  <Teleport to="body">
    <div v-if="showQuickBindDialog && quickBindPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="closeQuickBind">
      <div class="dialog-container w-full max-w-lg max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex items-center justify-between mb-2">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('plugins.quickBind.title') }}</h3>
          <button @click="closeQuickBind" class="text-gray-500 dark:text-content-secondary hover:text-gray-700 dark:hover:text-content-primary">
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">
          {{ t('plugins.quickBind.desc', { name: quickBindPlugin.name }) }}
        </p>

        <div v-if="quickBindPlugin.versions.length > 1 || (quickBindPlugin.versions.length > 0 && quickBindPlugin.versions[0].units.length > 1)" class="mb-2 space-y-2 p-3 bg-gray-50 dark:bg-surface-layer rounded-[6px]">
          <div v-if="quickBindPlugin.versions.length > 1">
            <label class="block text-xs font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('linker.selectVersion') }}</label>
            <select v-model="quickBindVersionIdx" class="w-full select-field">
              <option v-for="(v, i) in quickBindPlugin.versions" :key="v.version_id" :value="i">v{{ v.version }}</option>
            </select>
          </div>
          <div v-if="quickBindPlugin.versions[quickBindVersionIdx]?.units.length > 1">
            <label class="block text-xs font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('linker.selectUnitLabel') }}</label>
            <select v-model="quickBindUnitIdx" class="w-full select-field">
              <option v-for="(u, i) in quickBindPlugin.versions[quickBindVersionIdx]?.units" :key="u.unit_id" :value="i">{{ u.name }}</option>
            </select>
          </div>
        </div>

        <div v-if="quickBindProjects.length === 0" class="text-center py-6">
          <p class="text-sm text-gray-500 dark:text-content-secondary">{{ t('plugins.quickBind.noProjects') }}</p>
        </div>
        <div v-else class="flex-1 overflow-y-auto space-y-1 mb-2">
          <div class="text-xs font-medium text-gray-500 dark:text-content-secondary mb-2">{{ t('plugins.quickBind.selectProjects') }}</div>
          <div
            v-for="project in quickBindProjects"
            :key="project.project_id"
            @click="toggleQuickBindProject(project.project_id)"
            :class="['flex items-center gap-2 p-2 rounded-[4px] cursor-pointer transition-colors', quickBindSelectedProjectIds.has(project.project_id) ? 'bg-primary-50 dark:bg-surface-hover ring-1 ring-primary-300 dark:ring-brand-primary' : 'hover:bg-gray-50 dark:hover:bg-surface-layer']"
          >
            <div class="w-4 h-4 rounded-[4px] border flex-shrink-0 flex items-center justify-center" :class="quickBindSelectedProjectIds.has(project.project_id) ? 'bg-primary-600 border-primary-600' : 'border-gray-300 dark:border-surface-border'">
              <svg v-if="quickBindSelectedProjectIds.has(project.project_id)" class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <div class="min-w-0 flex-1">
              <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ project.name }}</div>
              <div class="text-xs text-gray-500 dark:text-content-secondary">{{ project.godot_version }}</div>
            </div>
            <span v-if="quickBindBoundProjectIds.has(project.project_id)" class="text-xs text-green-600 dark:text-green-400 flex-shrink-0 font-medium">✓ {{ t('projects.bound') }}</span>
            <span v-if="isCompatWarning(quickBindPlugin, project)" class="text-xs text-orange-500 dark:text-orange-400 flex-shrink-0" :title="t('plugins.quickBind.compatWarning')">⚠</span>
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-3 border-t border-gray-200/60 dark:border-surface-border/40">
          <button @click="closeQuickBind" class="btn-secondary">{{ t('plugins.quickBind.bindLater') }}</button>
          <button
            @click="doQuickBind"
            :disabled="isQuickBinding || quickBindSelectedProjectIds.size === 0"
            class="btn-primary disabled:opacity-50"
          >
            {{ isQuickBinding ? t('plugins.quickBind.binding') : t('plugins.quickBind.bindSelected') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showVersionSwitchDialog && versionSwitchPlugin && versionSwitchBinding" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showVersionSwitchDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('plugins.versionSwitch.title') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-2">
          {{ t('plugins.versionSwitch.desc', { plugin: versionSwitchPlugin.name, project: linkerProjects.find(p => p.project_id === versionSwitchBinding?.project_id)?.name || versionSwitchBinding?.project_id }) }}
        </p>
        <div class="space-y-2 mb-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('plugins.versionSwitch.selectVersion') }}</label>
            <select v-model="versionSwitchVersionIdx" class="w-full select-field">
              <option v-for="(v, i) in versionSwitchPlugin.versions" :key="v.version_id" :value="i">v{{ v.version }}</option>
            </select>
          </div>
          <div v-if="versionSwitchPlugin.versions[versionSwitchVersionIdx]?.units.length > 1">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('plugins.versionSwitch.selectUnit') }}</label>
            <select v-model="versionSwitchUnitIdx" class="w-full select-field">
              <option v-for="(u, i) in versionSwitchPlugin.versions[versionSwitchVersionIdx]?.units" :key="u.unit_id" :value="i">{{ u.name }}</option>
            </select>
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showVersionSwitchDialog = false" class="btn-secondary">{{ t('common.cancel') }}</button>
          <button @click="doSwitchVersion" :disabled="isSwitchingVersion" class="btn-primary disabled:opacity-50">
            {{ isSwitchingVersion ? t('plugins.versionSwitch.switching') : t('plugins.versionSwitch.switchVersion') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showHarborConfigDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showHarborConfigDialog = false">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.configTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-secondary mb-3">{{ t('linker.configDesc') }}</p>
        <div v-if="exportSkippedLocal.length > 0" class="mb-3 p-2 rounded-[6px] bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800">
          <p class="text-sm font-medium text-yellow-700 dark:text-yellow-400">{{ t('linker.skippedLocalTitle') }}</p>
          <p class="text-xs text-yellow-600 dark:text-yellow-500 mt-1">{{ t('linker.skippedLocalDesc') }}</p>
          <div class="mt-1.5 flex flex-wrap gap-1">
            <span v-for="name in exportSkippedLocal" :key="name" class="inline-block px-1.5 py-0.5 text-[10px] bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 rounded-[4px]">{{ name }}</span>
          </div>
        </div>
        <div v-if="harborConfigContent" class="bg-gray-50 dark:bg-surface-layer rounded-[6px] p-3 text-xs font-mono text-gray-700 dark:text-content-secondary max-h-64 overflow-y-auto whitespace-pre-wrap break-all">{{ harborConfigContent }}</div>
        <div v-else class="text-sm text-gray-500 dark:text-content-secondary">{{ t('linker.configEmpty') }}</div>
        <div class="flex justify-between mt-4">
          <button
            v-if="harborConfigContent"
            @click="deleteHarborConfig"
            class="text-sm text-red-500 hover:text-red-600 dark:text-red-400 dark:hover:text-red-300"
          >{{ t('linker.deleteConfig') }}</button>
          <div v-else></div>
          <button @click="showHarborConfigDialog = false" class="btn-primary">{{ t('linker.close') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showSyncResultDialog && syncResult" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showSyncResultDialog = false">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('linker.syncResultTitle') }}</h3>
        <div class="space-y-2 mb-2">
          <div v-if="syncResult.imported > 0" class="text-sm text-green-600 dark:text-green-400">{{ t('linker.syncImported', { count: syncResult.imported }) }}</div>
          <div v-if="syncResult.bound > 0" class="text-sm text-blue-600 dark:text-brand-primary">{{ t('linker.syncBound', { count: syncResult.bound }) }}</div>
          <div v-if="syncResult.skipped > 0" class="text-sm text-yellow-600 dark:text-yellow-400">{{ t('linker.syncSkipped', { count: syncResult.skipped }) }}</div>
          <div v-if="syncResult.errors.length > 0" class="space-y-1">
            <p class="text-sm font-medium text-orange-600 dark:text-orange-400">{{ t('linker.syncWarnings') }}</p>
            <div v-for="err in syncResult.errors" :key="err" class="text-xs text-orange-500 dark:text-orange-400">{{ err }}</div>
          </div>
          <div v-if="syncResult.imported === 0 && syncResult.bound === 0 && syncResult.skipped === 0" class="text-sm text-gray-500 dark:text-content-secondary">{{ t('linker.syncNoChanges') }}</div>
        </div>
        <div class="flex justify-end">
          <button @click="showSyncResultDialog = false" class="btn-primary">{{ t('linker.close') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showUidConflictDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showUidConflictDialog = false; pendingBindAfterUidCheck = null">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <div class="flex items-center gap-2 mb-2">
          <div class="w-10 h-10 rounded-full bg-yellow-100 dark:bg-yellow-900/30 flex items-center justify-center shrink-0">
            <svg class="w-5 h-5 text-yellow-600 dark:text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
          </div>
          <div>
            <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('linker.uidConflictTitle') }}</h3>
            <p class="text-sm text-gray-500 dark:text-content-secondary mt-0.5">{{ t('linker.uidConflictDesc') }}</p>
          </div>
        </div>

        <div class="space-y-2 mb-2 max-h-36 overflow-y-auto">
          <div v-for="conflict in uidConflicts" :key="conflict.plugin_id" class="p-2 bg-yellow-50 dark:bg-yellow-900/10 rounded-[6px] border border-yellow-200 dark:border-yellow-800">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ conflict.plugin_name }}</span>
              <span class="text-xs px-1.5 py-0.5 bg-yellow-200 dark:bg-yellow-800/50 text-yellow-700 dark:text-yellow-300 rounded-[4px]">{{ t('linker.uidConflictCount', { count: conflict.conflicting_uids.length }) }}</span>
            </div>
            <div v-if="conflict.conflicting_uids.length <= 5" class="mt-1 flex flex-wrap gap-1">
              <span v-for="uid in conflict.conflicting_uids" :key="uid" class="text-[10px] font-mono px-1 py-0.5 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 rounded-[4px]">{{ uid }}</span>
            </div>
          </div>
        </div>

        <div class="mb-2">
          <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('linker.uidConflictSolution') }}</h4>
          <div class="space-y-2">
            <button
              @click="handleBindWithCopyMode"
              class="w-full text-left p-3 rounded-[6px] border-2 transition-colors"
              :class="mountStrategyDisplay !== 'Copy' ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40'"
            >
              <div class="flex items-center gap-2">
                <svg class="w-4 h-4 text-primary-600 dark:text-brand-primary shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('linker.uidConflictSolutionCopy') }}</span>
              </div>
              <p class="text-xs text-gray-500 dark:text-content-muted mt-1 ml-6">{{ t('linker.uidConflictSolutionCopyDesc') }}</p>
            </button>
            <button
              @click="showUidConflictDialog = false; pendingBindAfterUidCheck?.()"
              class="w-full text-left p-3 rounded-[6px] border border-gray-200/60 dark:border-surface-border/40 hover:border-red-300 dark:hover:border-red-700 transition-colors"
            >
              <div class="flex items-center gap-2">
                <svg class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
                <span class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('linker.uidConflictSolutionForce') }}</span>
              </div>
              <p class="text-xs text-gray-500 dark:text-content-muted mt-1 ml-6">{{ t('linker.uidConflictSolutionForceDesc') }}</p>
            </button>
          </div>
        </div>

        <div class="flex justify-end">
          <button @click="showUidConflictDialog = false; pendingBindAfterUidCheck = null" class="btn-secondary">{{ t('common.cancel') }}</button>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Global Upgrade Dialog -->
  <GlobalUpgradeDialog
    :visible="showGlobalUpgradeDialog"
    :pluginId="globalUpgradePluginId"
    :pluginName="globalUpgradePluginName"
    @update:visible="showGlobalUpgradeDialog = $event"
    @close="showGlobalUpgradeDialog = false"
    @upgraded="loadPlugins()"
  />

  <ContextMenu
    :visible="pluginContextMenu.visible.value"
    :x="pluginContextMenu.x.value"
    :y="pluginContextMenu.y.value"
    :items="pluginContextMenu.items.value"
    @close="pluginContextMenu.close()"
  />
</template>