<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import { api } from '@/api'
import type { Engine, RemoteEngineVersion, EngineMirrorConfig, EngineDownloadProgress, EngineReleaseChannel, Project, EngineModulesInfo } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useFileManager } from '@/composables/useFileManager'
import { isOnline } from '@/composables/useNetworkStatus'
import { useContextMenu } from '@/composables/useContextMenu'
import type { ContextMenuEntry } from '@/composables/useContextMenu'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import SkeletonList from '@/components/SkeletonList.vue'
import ErrorState from '@/components/ErrorState.vue'
import EmptyState from '@/components/EmptyState.vue'
import EngineModulesPanel from '@/components/EngineModulesPanel.vue'

const toast = useToast()
const { t } = useI18n()
const { openInFileManager } = useFileManager()
const router = useRouter()
const route = useRoute()
const engines = ref<Engine[]>([])
const isLoading = ref(false)
const loadError = ref<string | null>(null)
const isDiscovering = ref(false)
const showAddDialog = ref(false)
const newEnginePath = ref('')
const newEngineName = ref('')
const isRegistering = ref(false)
const engineUrl = ref('')
const engineUrlName = ref('')
const isDownloadingFromUrl = ref(false)
const downloadTab = ref<'mirror' | 'url'>('mirror')
const showDeleteConfirm = ref(false)
const deleteAlsoFiles = ref(false)
const deleteTargetId = ref('')
let unlistenDiscover: UnlistenFn | null = null
let unlistenDownloadProgress: UnlistenFn | null = null
let unlistenAutoSetup: UnlistenFn | null = null

const searchQuery = ref('')
const debouncedSearchQuery = ref('')
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, (val) => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  searchDebounceTimer = setTimeout(() => {
    debouncedSearchQuery.value = val
  }, 300)
})
const filterType = ref<string>('all')
const engineHealthMap = ref<Map<string, boolean>>(new Map())

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
const failedDownloads = ref<Map<string, string>>(new Map())
const expandedReleaseVersion = ref<string>('')
const openMenuId = ref<string>('')
const collapsedGroups = ref<Set<string>>(new Set())
const showReDownloadConfirm = ref(false)
const reDownloadTarget = ref<RemoteEngineVersion | null>(null)
const expandedModulesEngineId = ref<string>('')
const engineModulesMap = ref<Map<string, EngineModulesInfo>>(new Map())

useDialogEscape(showAddDialog)
useDialogEscape(showRenameDialog)
useDialogEscape(showDownloadDialog)

const engineContextMenu = useContextMenu()

const showEngineContextMenu = (event: MouseEvent, engine: Engine) => {
  engineContextMenu.show(event, [
    {
      label: t('engines.contextMenu.launch'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>',
      action: () => launchEngine(engine.engine_id),
      disabled: engineHealthMap.value.get(engine.engine_id) === false || isLaunchingEngine,
    },
    {
      label: t('engines.rename'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>',
      action: () => openRenameDialog(engine),
    },
    { separator: true },
    {
      label: t('engines.contextMenu.manageModules'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>',
      action: () => toggleModulesPanel(engine.engine_id),
    },
    { separator: true },
    {
      label: t('engines.contextMenu.openInFileManager'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>',
      action: () => openInFileManager(engine.path),
    },
    {
      label: t('engines.contextMenu.removeEngine'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>',
      action: () => confirmRemoveEngine(engine.engine_id),
      danger: true,
    },
  ] as ContextMenuEntry[])
}

onMounted(async () => {
  const [, activeListResult] = await Promise.allSettled([
    loadEngines(),
    api.getActiveDownloads()
  ])
  loadProjects()
  if (engines.value.length === 0) {
    isDiscovering.value = true
  }
  if (activeListResult.status === 'fulfilled') {
    const newMap = new Map<string, EngineDownloadProgress>()
    for (const progress of activeListResult.value) {
      const key = `${progress.version}_${progress.variant}`
      newMap.set(key, progress)
    }
    activeDownloads.value = newMap
  }
  unlistenDiscover = await listen('engines-discovered', () => {
    isDiscovering.value = false
    loadEngines()
  })
  unlistenAutoSetup = await listen('auto-setup-complete', () => {
    isDiscovering.value = false
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
  if (route.query.action === 'register') {
    await nextTick()
    showAddDialog.value = true
    router.replace({ path: '/engines' })
  }
})

onUnmounted(() => {
  if (unlistenDiscover) {
    unlistenDiscover()
  }
  if (unlistenDownloadProgress) {
    unlistenDownloadProgress()
  }
  if (unlistenAutoSetup) {
    unlistenAutoSetup()
  }
  document.removeEventListener('click', handleGlobalClick)
})

const filteredEngines = computed(() => {
  return engines.value.filter(engine => {
    const matchesSearch = debouncedSearchQuery.value === '' ||
      engine.name.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase()) ||
      engine.version.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase()) ||
      engine.path.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase())

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

const latestStableKey = computed(() => {
  const stables = remoteVersions.value.filter(v => v.channel === 'Stable' && v.variant === 'standard')
  if (stables.length === 0) return ''
  stables.sort((a, b) => {
    if (a.major !== b.major) return b.major - a.major
    if (a.minor !== b.minor) return b.minor - a.minor
    return b.patch - a.patch
  })
  return `${stables[0].version}_${stables[0].variant}`
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
    case 'Rc': return 'bg-blue-100 text-blue-800 dark:bg-surface-hover dark:text-brand-primary'
    case 'Beta': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400'
    case 'Alpha': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'
    case 'Dev': return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
    default: return 'bg-gray-100 text-gray-800 dark:bg-surface-hover dark:text-content-secondary'
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

const formatEta = (seconds: number) => {
  if (seconds < 60) return `${seconds}s`
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  if (m < 60) return `${m}m ${s}s`
  const h = Math.floor(m / 60)
  return `${h}h ${m % 60}m`
}

const formatProgressMessage = (progress: EngineDownloadProgress) => {
  switch (progress.stage) {
    case 'downloading':
      if (progress.total_bytes > 0) {
        return t('engines.download.downloadProgress', {
          downloaded: formatFileSize(progress.downloaded_bytes),
          total: formatFileSize(progress.total_bytes),
          speed: formatFileSize(progress.speed)
        })
      }
      return t('engines.download.downloadProgressNoTotal', {
        downloaded: formatFileSize(progress.downloaded_bytes),
        speed: formatFileSize(progress.speed)
      })
    case 'extracting':
      return t('engines.download.extractingProgress', {
        current: formatFileSize(progress.downloaded_bytes),
        total: formatFileSize(progress.total_bytes)
      })
    case 'parsing':
      return t('engines.download.parsingProgress')
    case 'error':
      return t('engines.download.errorProgress')
    case 'complete':
      return t('engines.download.downloadComplete')
    default:
      return progress.message
  }
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
  loadError.value = null
  try {
    const result = await api.getEngines()
    engines.value = result
    await checkAllEngineHealth()
  } catch (error) {
    loadError.value = String(error)
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

const discoverEngines = async () => {
  isLoading.value = true
  isDiscovering.value = true
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
    isDiscovering.value = false
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

const isLaunchingEngine = ref(false)
const projects = ref<Project[]>([])

const loadProjects = async () => {
  try {
    projects.value = await api.getProjects()
  } catch { /* ignore */ }
}

const getRecentProjectForEngine = (engineId: string): Project | null => {
  const matchingProjects = projects.value
    .filter(p => p.last_used_engine_id === engineId)
    .sort((a, b) => (b.last_opened_at || b.updated_at).localeCompare(a.last_opened_at || a.updated_at))
  return matchingProjects[0] || null
}

const launchRecentProject = async (engineId: string) => {
  const project = getRecentProjectForEngine(engineId)
  if (!project) {
    toast.warning(t('engines.noRecentProject'))
    return
  }
  if (isLaunchingEngine.value) return
  isLaunchingEngine.value = true
  try {
    await api.launchEngine(engineId, project.path, project.project_id)
    toast.success(t('engines.launchProjectSuccess', { name: project.name }))
  } catch (error) {
    toast.error(t('engines.launchFailed', { error }))
  } finally {
    isLaunchingEngine.value = false
  }
}

const launchEngine = async (engineId: string) => {
  if (isLaunchingEngine.value) return
  isLaunchingEngine.value = true
  try {
    await api.launchEngine(engineId)
    toast.success(t('engines.launchSuccess'))
  } catch (error) {
    toast.error(t('engines.launchFailed', { error }))
  } finally {
    isLaunchingEngine.value = false
  }
}

const downloadEngineFromUrl = async () => {
  if (!isOnline.value) {
    toast.warning(t('common.offlineNotice'))
    return
  }
  if (!engineUrl.value) {
    toast.warning(t('engines.urlDownload.enterUrl'))
    return
  }
  const urlLower = engineUrl.value.toLowerCase()
  if (!urlLower.endsWith('.zip') && !urlLower.includes('.zip?') && !urlLower.includes('.zip/')) {
    toast.warning(t('engines.urlDownload.zipHint'))
    return
  }
  isDownloadingFromUrl.value = true
  try {
    const result = await api.downloadEngineFromUrl(engineUrl.value, engineUrlName.value)
    if (result.success && result.engine) {
      toast.success(t('engines.registerSuccess', { name: result.engine.name }))
      showDownloadDialog.value = false
      engineUrl.value = ''
      engineUrlName.value = ''
      await loadEngines()
    } else if (result.cancelled) {
      toast.info(t('engines.download.downloadCancelled'))
    } else if (result.error) {
      toast.error(result.error)
    }
  } catch (error) {
    toast.error(t('engines.urlDownload.failed', { error }))
  } finally {
    isDownloadingFromUrl.value = false
  }
}

const openRenameDialog = (engine: Engine) => {
  renameEngineId.value = engine.engine_id
  renameInput.value = engine.name
  showRenameDialog.value = true
}

const handleRelocateEngine = async (engineId: string) => {
  const selected = await open({ directory: true, multiple: false, title: t('engines.relocateTitle') })
  if (!selected) return
  const newPath = typeof selected === 'string' ? selected : (selected as string[])[0]
  if (!newPath) return
  try {
    await api.relocateEngine(engineId, newPath)
    toast.success(t('engines.relocateSuccess'))
    await loadEngines()
  } catch (e: any) {
    toast.error(t('engines.relocateFailed', { error: e?.toString() || e }) || `重定位失败: ${e}`)
  }
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

const openDownloadDialog = async () => {
  showDownloadDialog.value = true
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
      const timeMatch = errMsg.match(/RATE_LIMITED:(.+)/)
      if (timeMatch) {
        toast.error(t('engines.download.rateLimitErrorWithTime', { time: timeMatch[1] }))
      } else {
        toast.error(t('engines.download.rateLimitError'))
      }
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
  if (!isOnline.value) {
    toast.warning(t('common.offlineNotice'))
    return
  }
  const dlKey = `${version.version}_${version.variant}`
  if (activeDownloads.value.has(dlKey)) {
    toast.info(t('engines.download.alreadyDownloading'))
    return
  }
  if (version.is_installed) {
    reDownloadTarget.value = version
    showReDownloadConfirm.value = true
    return
  }
  doStartDownload(version)
}

const doStartDownload = (version: RemoteEngineVersion) => {
  const dlKey = `${version.version}_${version.variant}`
  const newMap = new Map(activeDownloads.value)
  newMap.set(dlKey, {
    version: version.version,
    variant: version.variant,
    stage: 'downloading',
    downloaded_bytes: 0,
    total_bytes: 0,
    progress: 0,
    message: t('engines.download.downloading'),
    speed: 0,
    eta: 0
  })
  activeDownloads.value = newMap
  const failedMap = new Map(failedDownloads.value)
  failedMap.delete(dlKey)
  failedDownloads.value = failedMap

  api.downloadEngine(version).then(result => {
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
      loadEngines()
    } else if (result.error) {
      toast.error(t('engines.download.downloadFailed', { error: result.error }))
      const fm = new Map(failedDownloads.value)
      fm.set(dlKey, result.error)
      failedDownloads.value = fm
    }
  }).catch(error => {
    toast.error(t('engines.download.downloadFailed', { error }))
    const fm = new Map(failedDownloads.value)
    fm.set(dlKey, String(error))
    failedDownloads.value = fm
  }).finally(() => {
    const cleanupMap = new Map(activeDownloads.value)
    cleanupMap.delete(dlKey)
    activeDownloads.value = cleanupMap
  })
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

const toggleModulesPanel = async (engineId: string) => {
  if (expandedModulesEngineId.value === engineId) {
    expandedModulesEngineId.value = ''
    return
  }
  expandedModulesEngineId.value = engineId
  if (!engineModulesMap.value.has(engineId)) {
    try {
      const info = await api.getEngineModules(engineId)
      const newMap = new Map(engineModulesMap.value)
      newMap.set(engineId, info)
      engineModulesMap.value = newMap
    } catch { /* ignore */ }
  }
}

const getModuleCountBadge = (engineId: string): string => {
  const info = engineModulesMap.value.get(engineId)
  if (!info) return ''
  const nonEditor = info.modules.filter(m => m.module_type !== 'Editor')
  const installed = nonEditor.filter(m => m.is_installed).length
  const total = nonEditor.length
  if (total === 0) return ''
  return `${installed}/${total}`
}

const hasMissingModules = (engineId: string): boolean => {
  const info = engineModulesMap.value.get(engineId)
  if (!info) return false
  return info.modules.some(m => m.module_type !== 'Editor' && !m.is_installed)
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="shrink-0 space-y-2 pb-2">
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
      <h1 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('engines.title') }}</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="discoverEngines"
          :disabled="isLoading"
          class="btn-secondary disabled:opacity-50 text-sm"
        >
          {{ t('engines.discover') }}
        </button>
        <button
          @click="openDownloadDialog"
          class="px-4 py-2 border border-primary-600 text-primary-600 dark:text-brand-primary bg-white dark:bg-surface-card rounded-btn hover:bg-primary-50 dark:hover:bg-surface-hover transition-colors text-sm inline-flex items-center gap-1.5"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          {{ t('engines.download.title') }}
        </button>
        <button
          @click="showAddDialog = true"
          class="btn-primary text-sm"
        >
          {{ t('engines.register') }}
        </button>
      </div>
    </div>

    <div v-if="activeDownloads.size > 0 && !showDownloadDialog" class="fixed bottom-4 right-4 z-30 bg-white dark:bg-surface-card rounded-island border border-blue-200 dark:border-surface-border/50 p-3 w-80">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm font-medium text-blue-800 dark:text-content-secondary">
          {{ t('engines.download.downloading') }} ({{ activeDownloads.size }})
        </span>
        <button
          @click="openDownloadDialog"
          class="text-xs text-primary-600 dark:text-brand-primary hover:underline"
        >
          {{ t('engines.download.title') }}
        </button>
      </div>
      <div class="space-y-2">
        <div v-for="[key, progress] in activeDownloads" :key="key">
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs text-blue-700 dark:text-content-secondary">v{{ progress.version }}{{ progress.variant === 'mono' ? ' (.NET)' : '' }} - {{ formatProgressMessage(progress) }}</span>
            <div class="flex items-center gap-2">
              <span v-if="progress.speed > 0" class="text-xs text-blue-600 dark:text-brand-primary">{{ formatFileSize(progress.speed) }}/s</span>
              <span class="text-xs text-blue-600 dark:text-brand-primary">{{ progress.progress.toFixed(1) }}%</span>
              <button
                @click="cancelDownload(progress.version, progress.variant)"
                class="text-xs text-red-600 dark:text-red-400 hover:underline"
              >
                {{ t('engines.download.cancel') }}
              </button>
            </div>
          </div>
          <div class="w-full bg-blue-200 dark:bg-surface-border rounded-full h-1.5">
            <div class="bg-blue-600 dark:bg-brand-primary h-1.5 rounded-full transition-all duration-300" :style="{ width: `${progress.progress}%` }"></div>
          </div>
        </div>
      </div>
    </div>

    <div class="border-b border-gray-200/60 dark:border-surface-border/40 pb-2">
      <div class="flex flex-col lg:flex-row gap-2">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('engines.search')"
            class="input-field"
          />
        </div>
        <div class="flex gap-1.5 items-center">
          <select
            v-model="filterType"
            class="select-field"
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

    <div v-if="isLoading" class="py-4">
      <SkeletonList :count="4" type="engine" />
    </div>

    <ErrorState
      v-else-if="loadError"
      :title="t('common.loadFailed', { error: '' })"
      :description="loadError"
      :retryLabel="t('common.retry')"
      @retry="loadEngines"
    />

    <div v-else-if="isDiscovering && engines.length === 0" class="text-center py-16">
      <div class="animate-spin rounded-full h-10 w-10 border-2 border-primary-600 border-t-transparent mx-auto"></div>
      <h3 class="mt-4 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('engines.discovering') }}</h3>
      <p class="mt-1 text-xs text-gray-500 dark:text-content-muted">{{ t('autoSetup.pleaseWait') }}</p>
    </div>

    <EmptyState
      v-else-if="engines.length === 0"
      :title="t('engines.empty')"
      :description="t('engines.emptyDesc')"
      :actionLabel="t('engines.discover')"
      @action="discoverEngines"
      :shortcuts="[
        { key: 'Ctrl+K', description: t('commandPalette.title') },
      ]"
    />

    <div v-else-if="filteredEngines.length === 0" class="text-center py-12">
      <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('engines.noMatchingEngines') }}</p>
    </div>

    <div v-else class="flex-1 min-h-0 border border-gray-200 dark:border-surface-border/50 rounded overflow-hidden">
      <div class="overflow-x-hidden h-full overflow-y-auto">
        <table class="w-full min-w-[800px]">
          <tbody class="divide-y divide-gray-100 dark:divide-surface-border/40">
            <template v-for="engine in filteredEngines" :key="engine.engine_id">
            <tr
              class="hover:bg-gray-50 dark:hover:bg-surface-hover/50 transition-colors"
              @contextmenu="showEngineContextMenu($event, engine)"
            >
              <td class="px-3 py-2 whitespace-nowrap">
                <div class="flex items-center gap-2">
                  <div class="w-7 h-7 rounded bg-primary-100 dark:bg-surface-hover flex items-center justify-center">
                    <svg class="w-4 h-4 text-primary-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                  </div>
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-gray-900 dark:text-content-primary text-sm">
                        {{ engine.name }}
                      </span>
                    </div>
                    <span class="text-xs text-gray-500 dark:text-content-muted">v{{ engine.version }}</span>
                    <span
                      v-if="engine.version.toLowerCase().includes('mono')"
                      class="px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-surface-hover dark:text-content-secondary"
                    >
                      .NET
                    </span>
                    <span
                      v-if="getModuleCountBadge(engine.engine_id)"
                      :class="[
                        'px-1.5 py-0.5 rounded text-xs font-medium cursor-pointer',
                        hasMissingModules(engine.engine_id)
                          ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400'
                          : 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
                      ]"
                      @click="toggleModulesPanel(engine.engine_id)"
                    >
                      {{ getModuleCountBadge(engine.engine_id) }}
                    </span>
                  </div>
                </div>
              </td>
              <td class="px-3 py-2 whitespace-nowrap">
                <span class="badge badge-neutral">
                  {{ engine.engine_type === 'Godot4' ? 'Godot 4' : engine.engine_type === 'Godot3' ? 'Godot 3' : t('engines.unknown') }}
                </span>
              </td>
              <td class="px-3 py-2 whitespace-nowrap">
                <span
                  v-if="engineHealthMap.get(engine.engine_id) === true"
                  class="badge badge-success"
                >
                  {{ t('engines.healthy') }}
                </span>
                <span
                  v-else-if="engineHealthMap.get(engine.engine_id) === false"
                  class="badge badge-error"
                >
                  {{ t('engines.unhealthy') }}
                </span>
                <span v-else class="badge badge-neutral">{{ t('engines.checking') }}</span>
              </td>
              <td class="px-3 py-2">
                <span
                  class="text-sm text-primary-600 dark:text-brand-primary hover:underline cursor-pointer truncate max-w-xs block"
                  :title="engine.path"
                  @click="openInFileManager(engine.path)"
                >
                  {{ engine.path }}
                </span>
              </td>
              <td class="px-3 py-2 whitespace-nowrap">
                <div class="flex items-center justify-end gap-1">
                  <button
                    @click="toggleModulesPanel(engine.engine_id)"
                    :class="[
                      'p-2 rounded transition-colors',
                      expandedModulesEngineId === engine.engine_id
                        ? 'bg-primary-50 dark:bg-surface-hover text-primary-600 dark:text-brand-primary'
                        : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-surface-layer'
                    ]"
                    :title="t('engines.modules.title')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                    </svg>
                  </button>
                  <button
                    @click="launchEngine(engine.engine_id)"
                    :disabled="engineHealthMap.get(engine.engine_id) === false || isLaunchingEngine"
                    class="text-primary-600 dark:text-brand-primary hover:text-primary-800 dark:hover:text-brand-primary p-2.5 rounded hover:bg-primary-50 dark:hover:bg-surface-hover transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    :title="t('engines.launch')"
                  >
                    <svg v-if="isLaunchingEngine" class="h-6 w-6 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    <svg v-else class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  </button>
                  <button
                    v-if="getRecentProjectForEngine(engine.engine_id)"
                    @click="launchRecentProject(engine.engine_id)"
                    :disabled="isLaunchingEngine"
                    class="text-green-600 dark:text-green-400 hover:text-green-800 dark:hover:text-green-300 p-2.5 rounded hover:bg-green-50 dark:hover:bg-green-900/20 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    :title="t('engines.launchRecentProject', { name: getRecentProjectForEngine(engine.engine_id)!.name })"
                  >
                    <svg v-if="isLaunchingEngine" class="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    <svg v-else class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                  </button>
                  <div class="engine-menu-wrapper" style="position: relative; display: inline-block">
                    <button
                      @click="toggleEngineMenu(engine.engine_id)"
                      class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 p-2 rounded hover:bg-gray-100 dark:hover:bg-surface-layer transition-colors"
                      :title="t('engines.moreActions')"
                    >
                      <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                      </svg>
                    </button>
                    <div
                      v-if="openMenuId === engine.engine_id"
                      class="absolute right-0 top-full mt-1 bg-white dark:bg-surface-hover rounded-[6px] border border-gray-200 dark:border-surface-border/50 py-1 z-20 min-w-[140px]"
                    >
                      <button
                        @click="openRenameDialog(engine); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
                        {{ t('engines.rename') }}
                      </button>
                      <button
                        @click="handleRelocateEngine(engine.engine_id); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" /></svg>
                        {{ t('engines.relocate') }}
                      </button>
                      <button
                        @click="openInFileManager(engine.path); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
                        {{ t('engines.openInFileManager') }}
                      </button>
                      <hr class="my-1 border-gray-200/60 dark:border-surface-border/40" />
                      <button
                        @click="confirmRemoveEngine(engine.engine_id); openMenuId = ''"
                        class="w-full text-left px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2"
                      >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                        {{ t('engines.deleteEngine') }}
                      </button>
                    </div>
                  </div>
                </div>
              </td>
            </tr>
            <tr v-if="expandedModulesEngineId === engine.engine_id">
              <td colspan="5" class="px-3 py-2 bg-gray-50 dark:bg-surface-hover/30">
                <EngineModulesPanel :engine-id="engine.engine_id" />
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
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="dialog-title">{{ t('engines.registerTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-3">
          {{ t('engines.registerDesc') }}
        </p>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('engines.engineName') }}</label>
            <input
              v-model="newEngineName"
              type="text"
              :placeholder="t('engines.engineNamePlaceholder')"
              class="input-field"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('engines.enginePath') }}</label>
            <div class="flex gap-2">
              <input
                v-model="newEnginePath"
                type="text"
                readonly
                :placeholder="t('engines.enginePathPlaceholder')"
                class="input-field bg-gray-50 dark:bg-surface-hover"
              />
              <button
                @click="selectEnginePath"
                class="btn-secondary text-sm whitespace-nowrap"
              >
                {{ t('projects.browse') }}
              </button>
            </div>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-4">
          <button
            @click="showAddDialog = false; newEnginePath = ''; newEngineName = ''"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="registerEngine"
            :disabled="isRegistering || !newEnginePath"
            class="btn-primary disabled:opacity-50"
          >
            {{ isRegistering ? t('engines.registering') : t('engines.register') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showDownloadDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="handleDownloadDialogClose">
      <div class="dialog-container w-full max-w-3xl max-h-[85vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center pb-2 mb-2 border-b border-gray-200 dark:border-surface-border/50">
          <h3 class="dialog-title mb-0">{{ t('engines.download.title') }}</h3>
          <button @click="handleDownloadDialogClose" class="text-sm text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            {{ t('common.close') }}
          </button>
        </div>

        <div class="px-3 pb-2 space-y-2">
          <div class="flex border-b border-gray-200/60 dark:border-surface-border/40 mb-2">
            <button
              @click="downloadTab = 'mirror'"
              :class="downloadTab === 'mirror' ? 'tab-item-active' : 'tab-item'"
            >
              {{ t('engines.download.mirrorTab') }}
            </button>
            <button
              @click="downloadTab = 'url'"
              :class="downloadTab === 'url' ? 'tab-item-active' : 'tab-item'"
            >
              {{ t('engines.download.urlTab') }}
            </button>
          </div>

          <template v-if="downloadTab === 'mirror'">
          <div class="flex gap-2 items-end">
            <div class="flex-1">
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('engines.download.mirrorTab') }}</label>
              <select
                v-model="selectedMirrorId"
                @change="onMirrorChange"
                :disabled="isFetchingVersions"
                class="input-field"
              >
                <option v-for="mirror in mirrorConfigs" :key="mirror.id" :value="mirror.id" :disabled="!mirror.enabled">
                  {{ mirror.name }}{{ !mirror.enabled ? ` (${t('engines.download.disabled')})` : '' }}
                </option>
              </select>
            </div>
            <button
              @click="fetchRemoteVersions(true)"
              :disabled="isFetchingVersions"
              class="px-4 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-btn hover:bg-gray-50 dark:hover:bg-surface-layer text-sm whitespace-nowrap disabled:opacity-50"
            >
              {{ isFetchingVersions ? t('engines.download.fetching') : t('engines.download.refresh') }}
            </button>
          </div>

          <div class="flex gap-2">
            <div class="flex-1">
              <input
                v-model="downloadSearchQuery"
                type="text"
                :placeholder="t('engines.download.searchPlaceholder')"
                class="input-field"
              />
            </div>
            <select
              v-model="downloadChannelFilter"
              class="input-field"
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
              class="input-field"
            >
              <option value="all">{{ t('engines.download.allVariants') }}</option>
              <option value="standard">{{ t('engines.download.variantStandard') }}</option>
              <option value="mono">{{ t('engines.download.variantMono') }}</option>
            </select>
            <label class="flex items-center gap-1.5 text-sm text-gray-600 dark:text-content-muted cursor-pointer whitespace-nowrap">
              <input type="checkbox" v-model="hideInstalled" class="checkbox-field" />
              {{ t('engines.download.hideInstalled') }}
            </label>
          </div>

          <div v-if="activeDownloads.size > 0" class="space-y-2">
            <div v-for="[key, progress] in activeDownloads" :key="key" class="bg-blue-50 dark:bg-surface-hover rounded p-2">
              <div class="flex justify-between items-center mb-1">
                <span class="text-sm font-medium text-blue-800 dark:text-content-secondary">v{{ progress.version }}{{ progress.variant === 'mono' ? ' (.NET)' : '' }} - {{ formatProgressMessage(progress) }}</span>
                <span class="text-xs text-blue-600 dark:text-brand-primary">{{ progress.progress.toFixed(1) }}%</span>
              </div>
              <div class="w-full bg-blue-200 dark:bg-surface-border rounded-full h-2">
                <div class="bg-blue-600 dark:bg-brand-primary h-2 rounded-full transition-all duration-300" :style="{ width: `${progress.progress}%` }"></div>
              </div>
              <div class="flex justify-between text-xs text-blue-500 dark:text-brand-primary mt-1">
                <span v-if="progress.total_bytes > 0">{{ formatFileSize(progress.downloaded_bytes) }} / {{ formatFileSize(progress.total_bytes) }}</span>
                <span v-else>{{ formatFileSize(progress.downloaded_bytes) }}</span>
                <span v-if="progress.speed > 0">{{ formatFileSize(progress.speed) }}/s{{ progress.eta > 0 ? ` · ${formatEta(progress.eta)}` : '' }}</span>
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
          </template>

          <template v-if="downloadTab === 'url'">
            <div class="space-y-3 py-1">
              <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('engines.urlDownload.desc') }}</p>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('engines.urlDownload.urlLabel') }}</label>
                <input
                  v-model="engineUrl"
                  type="text"
                  :placeholder="t('engines.urlDownload.urlPlaceholder')"
                  class="input-field"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('engines.engineName') }}</label>
                <input
                  v-model="engineUrlName"
                  type="text"
                  :placeholder="t('engines.urlDownload.namePlaceholder')"
                  class="input-field"
                />
              </div>
              <div class="flex justify-end">
                <button
                  @click="downloadEngineFromUrl"
                  :disabled="isDownloadingFromUrl || !engineUrl"
                  class="btn-primary disabled:opacity-50 text-sm"
                >
                  {{ isDownloadingFromUrl ? t('engines.registering') : t('engines.urlDownload.download') }}
                </button>
              </div>
            </div>
          </template>
        </div>

        <template v-if="downloadTab === 'mirror'">
        <div class="flex-1 overflow-y-auto px-3 pb-3">
          <div v-if="isFetchingVersions" class="flex justify-center py-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
          </div>

          <div v-else-if="remoteVersions.length === 0" class="text-center py-8">
            <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('engines.download.noVersions') }}</p>
          </div>

          <div v-else-if="filteredRemoteVersions.length === 0" class="text-center py-8">
            <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('engines.download.noMatchingVersions') }}</p>
          </div>

          <div v-else class="space-y-3">
            <div v-for="[groupKey, versions] in groupedRemoteVersions" :key="groupKey">
              <div
                class="sticky top-0 bg-white dark:bg-surface-card py-1.5 px-3 -mx-3 border-b border-gray-200/60 dark:border-surface-border/40 mb-2 z-10 cursor-pointer hover:bg-gray-50 dark:hover:bg-surface-hover/50 transition-colors"
                @click="toggleGroup(groupKey)"
              >
                <div class="flex items-center gap-2">
                  <svg class="w-3 h-3 text-gray-400 transition-transform" :class="{ '-rotate-90': collapsedGroups.has(groupKey) }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 9l-7 7-7-7" /></svg>
                  <span class="text-sm font-semibold text-gray-700 dark:text-content-secondary">Godot {{ groupKey }}</span>
                  <span class="text-xs text-gray-400 ml-2">{{ versions.length }} {{ t('engines.download.versionCount') }}</span>
                </div>
              </div>
              <div v-if="!collapsedGroups.has(groupKey)" class="space-y-2">
            <div v-for="[subKey, subVersions] in subGroupedVersions(versions)" :key="subKey">
              <div
                :class="[
                  'rounded-[4px] border transition-colors',
                  subVersions.every(v => v.is_installed)
                    ? 'bg-gray-50 dark:bg-surface-hover/30 border-gray-200/60 dark:border-surface-border/40'
                    : 'bg-white dark:bg-surface-card border-gray-200/60 dark:border-surface-border/40'
                ]"
              >
                <div
                  v-for="(version, vIdx) in subVersions"
                  :key="`${version.tag_name}_${version.variant}`"
                  :class="[
                    vIdx > 0 ? 'border-t border-gray-200 dark:border-surface-border/40' : ''
                  ]"
                >
                  <div class="flex items-center gap-3 p-2.5">
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="font-medium text-sm text-gray-900 dark:text-content-primary">v{{ version.version }}</span>
                        <span
                          v-if="latestStableKey === `${version.version}_${version.variant}`"
                          class="px-1.5 py-0.5 rounded text-xs font-medium bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-400"
                        >
                          {{ t('engines.download.latestStable') }}
                        </span>
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
                          class="px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-surface-hover dark:text-content-secondary"
                        >
                          .NET
                        </span>
                        <span
                          v-if="version.is_installed"
                          class="px-1.5 py-0.5 rounded text-xs font-medium bg-primary-100 text-primary-800 dark:bg-surface-hover dark:text-brand-primary"
                        >
                          {{ t('engines.download.installed') }}
                        </span>
                      </div>
                      <div class="flex items-center gap-3 mt-1 text-xs text-gray-500 dark:text-content-muted">
                        <span>{{ formatFileSize(version.file_size) }}</span>
                        <span>{{ formatDate(version.published_at) }}</span>
                        <button
                          v-if="version.release_url"
                          @click="openUrl(version.release_url)"
                          class="text-primary-600 dark:text-brand-primary hover:underline inline-flex items-center gap-0.5"
                        >
                          {{ t('engines.download.sourceLink') }}
                          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
                        </button>
                        <button
                          v-if="version.release_notes"
                          @click="expandedReleaseVersion = expandedReleaseVersion === `${version.version}_${version.variant}` ? '' : `${version.version}_${version.variant}`"
                          class="text-primary-600 dark:text-brand-primary hover:underline"
                        >
                          {{ expandedReleaseVersion === `${version.version}_${version.variant}` ? t('engines.download.hideNotes') : t('engines.download.showNotes') }}
                        </button>
                      </div>
                    </div>
                    <template v-if="activeDownloads.has(`${version.version}_${version.variant}`)">
                      <button
                        disabled
                        class="px-3 py-1.5 rounded text-xs font-medium whitespace-nowrap bg-blue-100 text-blue-600 dark:bg-surface-hover dark:text-brand-primary"
                      >
                        {{ t('engines.download.downloading') }}
                      </button>
                    </template>
                    <template v-else-if="failedDownloads.has(`${version.version}_${version.variant}`)">
                      <button
                        @click="doStartDownload(version)"
                        class="px-3 py-1.5 rounded text-xs font-medium transition-colors whitespace-nowrap bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/50"
                      >
                        {{ t('engines.download.retry') }}
                      </button>
                    </template>
                    <template v-else-if="version.is_installed">
                      <button
                        @click="startDownload(version)"
                        class="px-3 py-1.5 rounded text-xs font-medium transition-colors whitespace-nowrap bg-gray-100 text-gray-600 dark:bg-surface-hover dark:text-content-muted hover:bg-gray-200 dark:hover:bg-surface-layer"
                      >
                        {{ t('engines.download.reInstall') }}
                      </button>
                    </template>
                    <template v-else>
                      <button
                        @click="startDownload(version)"
                        class="px-3 py-1.5 text-xs font-medium transition-colors whitespace-nowrap btn-primary"
                      >
                        {{ t('engines.download.downloadAction') }}
                      </button>
                    </template>
                  </div>
                  <div
                    v-if="expandedReleaseVersion === `${version.version}_${version.variant}` && version.release_notes"
                    class="px-3 pb-3"
                  >
                    <div class="p-2 bg-gray-50 dark:bg-surface-hover/50 rounded text-xs text-gray-600 dark:text-content-secondary whitespace-pre-wrap max-h-40 overflow-y-auto">{{ version.release_notes }}</div>
                  </div>
                </div>
              </div>
            </div>
              </div>
            </div>
          </div>
        </div>
        </template>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showRenameDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRenameDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="dialog-title">{{ t('engines.renameTitle') }}</h3>
        <input
          v-model="renameInput"
          type="text"
          :placeholder="t('engines.engineNamePlaceholder')"
          class="input-field"
          @keyup.enter="saveRename"
        />
        <div class="flex justify-end space-x-3 mt-4">
          <button
            @click="showRenameDialog = false"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveRename"
            :disabled="!renameInput.trim()"
            class="btn-primary disabled:opacity-50"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('engines.deleteConfirm')"
      :description="t('engines.deleteConfirmDesc')"
      :confirm-text="t('common.confirm')"
      @confirm="onRemoveEngineConfirm"
    >
      <label class="flex items-center gap-2 mt-2 cursor-pointer">
        <input type="checkbox" v-model="deleteAlsoFiles" class="w-4 h-4 rounded-[3px] border border-gray-300 dark:border-surface-border text-red-500 focus:ring-2 focus:ring-red-500/20 bg-white dark:bg-surface-input cursor-pointer" />
        <span class="text-sm text-gray-600 dark:text-content-muted">{{ t('engines.deleteAlsoFiles') }}</span>
      </label>
      <p v-if="deleteAlsoFiles" class="mt-2 text-xs text-red-600 dark:text-red-400">{{ t('engines.deleteAlsoFilesWarning') }}</p>
    </ConfirmDialog>

    <ConfirmDialog
      v-model="showReDownloadConfirm"
      :title="t('engines.download.reInstallTitle')"
      :description="t('engines.download.reInstallDesc', { version: reDownloadTarget ? `v${reDownloadTarget.version}` : '' })"
      :confirm-text="t('engines.download.reInstall')"
      @confirm="reDownloadTarget && doStartDownload(reDownloadTarget); reDownloadTarget = null"
    />
  </Teleport>

  <ContextMenu
    :visible="engineContextMenu.visible.value"
    :x="engineContextMenu.x.value"
    :y="engineContextMenu.y.value"
    :items="engineContextMenu.items.value"
    @close="engineContextMenu.close()"
  />
</template>
