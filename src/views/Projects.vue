<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { api } from '@/api'
import type { Project, Engine, MovedProjectCandidate, ProjectBinding, Plugin, DriftReport, SyncPreview, SyncEnvironmentResult, VcsInfo, ModuleType, ProjectGroup, ProjectTemplate } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useBatchSelection } from '@/composables/useBatchSelection'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useAutoSetup } from '@/composables/useAutoSetup'
import { useFileManager } from '@/composables/useFileManager'
import { getStatusBadgeClass, getStatusInlineClass } from '@/utils/statusBadge'
import { preloadIcons, getIconUrl, getIconDebugInfo } from '@/composables/useIconCache'
import { useEngineLauncher } from '@/composables/useEngineLauncher'
import { useContextMenu } from '@/composables/useContextMenu'
import type { ContextMenuEntry } from '@/composables/useContextMenu'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import SkeletonList from '@/components/SkeletonList.vue'
import ErrorState from '@/components/ErrorState.vue'
import EmptyState from '@/components/EmptyState.vue'
import VcsStatusBadge from '@/components/VcsStatusBadge.vue'
import VcsPanel from '@/components/VcsPanel.vue'
import MissingModulesWarning from '@/components/MissingModulesWarning.vue'
import LockfilePanel from '@/components/LockfilePanel.vue'
import EnvironmentSnapshotPanel from '@/components/EnvironmentSnapshotPanel.vue'


const router = useRouter()
const route = useRoute()
const toast = useToast()
const { t } = useI18n()
const { isRunning: isAutoSetupRunning, stepMessage: autoSetupMessage, runAutoSetup } = useAutoSetup()
const { openInFileManager } = useFileManager()

const handleInstallModule = async (moduleType: any) => {
  if (!selectedProject.value) return
  const engineId = selectedProject.value.last_used_engine_id
  if (!engineId) {
    toast.warning(t('projects.specifyEngineFirst'))
    return
  }
  try {
    await api.installEngineModule(engineId, moduleType)
    toast.success(t('projects.installingModule', { type: moduleType }))
  } catch (e: any) {
    toast.error(String(e))
  }
}
const debugMode = ref(false)
const toggleDebug = (e: KeyboardEvent) => {
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    debugMode.value = !debugMode.value
  }
}
const projects = ref<Project[]>([])
const engines = ref<Engine[]>([])
const projectBindingMap = ref<Map<string, ProjectBinding[]>>(new Map())
const isLoading = ref(false)
const isRefreshing = ref(false)
const loadError = ref<string | null>(null)
const showScanDialog = ref(false)
const scanDirInput = ref('')
const showProjectDetail = ref(false)
const selectedProject = ref<Project | null>(null)
const detailTab = ref<'overview' | 'environment' | 'vcs'>('overview')
const showGroupDialog = ref(false)
const selectedGroupId = ref('')
const showCreateGroupDialog = ref(false)
const newGroupName = ref('')
const newGroupIcon = ref('')
const newGroupColor = ref('#6B7280')
const newGroupDescription = ref('')
const isCreatingGroup = ref(false)
const showGitDialog = ref(false)
const gitUrl = ref('')
const gitTargetDir = ref('')
const isCloningFromGit = ref(false)
const showAddMenu = ref(false)
const showAddProjectDialog = ref(false)
const addProjectMode = ref<'local' | 'git' | 'template'>('local')
const templateList = ref<ProjectTemplate[]>([])
const hubTemplateList = ref<any[]>([])
const selectedTemplateId = ref('')
const templateProjectName = ref('')
const templateTargetDir = ref('')
const isCreatingFromTemplate = ref(false)
const editingProjectId = ref<string | null>(null)
const projectMenuId = ref('')

const driftReport = ref<DriftReport | null>(null)
const isCheckingDrift = ref(false)
const syncPreview = ref<SyncPreview | null>(null)
const isSyncing = ref(false)
const showSyncPreview = ref(false)
const syncResult = ref<SyncEnvironmentResult | null>(null)
const driftMap = ref<Map<string, DriftReport>>(new Map())
const selectedSyncItems = ref<Set<string>>(new Set())

const vcsInfoMap = ref<Map<string, VcsInfo>>(new Map())
const projectMissingModulesMap = ref<Map<string, ModuleType[]>>(new Map())
const lockStatusMap = ref<Map<string, 'locked_verified' | 'locked_drifted' | 'not_locked'>>(new Map())

const toggleProjectMenu = (projectId: string) => {
  projectMenuId.value = projectMenuId.value === projectId ? '' : projectId
}



const handleGlobalClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.project-menu-wrapper')) {
    projectMenuId.value = ''
  }
  if (!target.closest('.relative')) {
    showAddMenu.value = false
  }

}
const projectBindings = ref<ProjectBinding[]>([])
const allPlugins = ref<Plugin[]>([])

const pluginNameMap = computed(() => {
  const map = new Map<string, string>()
  for (const p of allPlugins.value) {
    map.set(p.plugin_id, p.name)
  }
  return map
})

const getPluginName = (pluginId: string) => {
  return pluginNameMap.value.get(pluginId) || pluginId
}

const getPluginVersion = (pluginId: string) => {
  const plugin = allPlugins.value.find(p => p.plugin_id === pluginId)
  return plugin?.versions[0]?.version ? `v${plugin.versions[0].version}` : ''
}

const searchQuery = ref('')
const debouncedSearchQuery = ref('')
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, (val) => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  searchDebounceTimer = setTimeout(() => {
    debouncedSearchQuery.value = val
  }, 300)
})
const filterGroup = ref<string>('all')
const filterStatus = ref<string>('all')
const availableGroups = ref<ProjectGroup[]>([])
let unlisten: UnlistenFn | null = null
let unlistenFs: UnlistenFn | null = null
let unlistenAutoSetup: UnlistenFn | null = null

const sortBy = ref<string>('name')
const sortOrder = ref<string>('asc')
const hasScanDirs = ref(false)

const showBatchGroupDialog = ref(false)
const batchGroupId = ref('')

const isBatchApplying = ref(false)

const batchApplyChanges = async () => {
  const ids = Array.from(selectedProjectIds.value)
  if (ids.length === 0) return
  isBatchApplying.value = true
  try {
    const result = await api.batchApplyChanges(ids)
    const successCount = result.results.filter(r => r.success).length
    const failCount = result.results.filter(r => !r.success).length
    if (failCount > 0) {
      toast.warning(t('projects.batchApplyPartial', { success: successCount, failed: failCount }))
    } else {
      toast.success(t('projects.batchApplySuccess', { count: successCount }))
    }
    clearSelection()
  } catch (error) {
    toast.error(String(error))
  } finally {
    isBatchApplying.value = false
  }
}

const batchRemoveProjects = async () => {
  const ids = Array.from(selectedProjectIds.value)
  if (ids.length === 0) return
  confirmAction.value = {
    title: t('common.confirmDelete'),
    message: t('projects.deleteConfirm', { count: ids.length }),
    onConfirm: async () => {
      isBatchDeleting.value = true
      batchDeleteProgress.value = { current: 0, total: ids.length, message: t('projects.batchDeleting') }
      try {
        let successCount = 0
        let failCount = 0
        const concurrency = 3
        const chunks: string[][] = []
        for (let i = 0; i < ids.length; i += concurrency) {
          chunks.push(ids.slice(i, i + concurrency))
        }
        for (const chunk of chunks) {
          const results = await Promise.allSettled(
            chunk.map(id => api.batchRemoveProjects([id], deleteWithFiles.value))
          )
          for (const r of results) {
            if (r.status === 'fulfilled') {
              successCount += r.value.success_count
              failCount += r.value.failed_count
            } else {
              failCount++
            }
            batchDeleteProgress.value = {
              current: successCount + failCount,
              total: ids.length,
              message: t('projects.batchDeleting')
            }
          }
        }
        if (failCount > 0) {
          toast.warning(t('common.batchDeleteComplete', { success: successCount, failed: failCount }))
        } else {
          toast.success(deleteWithFiles.value ? t('projects.batchDeletedWithFiles', { count: successCount }) : t('common.batchDeleteSuccess', { count: successCount }))
        }
        clearSelection()
        await loadProjects()
      } catch (error) {
        toast.error(t('common.batchDeleteFailed', { error }))
      } finally {
        isBatchDeleting.value = false
        batchDeleteProgress.value = null
        deleteWithFiles.value = false
      }
    }
  }
  deleteWithFiles.value = false
  showConfirmDialog.value = true
}

const batchSetGroup = async () => {
  const ids = Array.from(selectedProjectIds.value)
  if (ids.length === 0) return
  batchGroupId.value = ''
  showBatchGroupDialog.value = true
}

const saveBatchGroup = async () => {
  const ids = Array.from(selectedProjectIds.value)
  try {
    await api.batchSetProjectGroup(ids, batchGroupId.value)
    toast.success(t('projects.batchGroupSuccess', { count: ids.length }))
  } catch (error) {
    toast.error(String(error))
  }
  showBatchGroupDialog.value = false
  clearSelection()
  await loadProjects()
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
  loadProjects()
  loadGroups()
  loadEngines()
  document.addEventListener('click', handleGlobalClick)
  document.addEventListener('keydown', toggleDebug)
  document.addEventListener('keydown', handleCtrlF)
  try {
    const settings = await api.getSettings()
    hasScanDirs.value = settings.scan_directories.length > 0
  } catch { /* ignore */ }
  unlisten = await listen('scan-complete', () => {
    loadProjects(true)
  })
  unlistenFs = await listen('project-fs-changed', async () => {
    try {
      const synced = await api.syncProjects()
      projects.value = synced
      await loadAllDrifts()
    } catch (error) {
      console.error('增量同步失败:', error)
    }
  })
  unlistenAutoSetup = await listen('auto-setup-complete', () => {
    loadProjects(true)
    loadEngines()
  })
  if (route.query.action === 'scan') {
    await nextTick()
    showScanDialog.value = true
    router.replace({ path: '/projects' })
  }
})

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick)
  document.removeEventListener('keydown', toggleDebug)
  if (unlisten) {
    unlisten()
  }
  if (unlistenFs) {
    unlistenFs()
  }
  if (unlistenAutoSetup) {
    unlistenAutoSetup()
  }
  document.removeEventListener('keydown', handleCtrlF)
})

const matchesSearch = (project: Project) =>
  debouncedSearchQuery.value === '' ||
  project.name.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase()) ||
  project.path.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase())

const getGroupById = (groupId: string): ProjectGroup | undefined => {
  return availableGroups.value.find(g => g.group_id === groupId)
}

const getGroupName = (groupId: string): string => {
  if (!groupId) return t('projects.ungrouped')
  return getGroupById(groupId)?.name || groupId
}

const getGroupIcon = (groupId: string): string => {
  if (!groupId) return ''
  return getGroupById(groupId)?.icon || ''
}

const getGroupColor = (groupId: string): string => {
  if (!groupId) return '#6B7280'
  return getGroupById(groupId)?.color || '#6B7280'
}

const groupedProjects = computed(() => {
  const groups: Record<string, Project[]> = {}

  const filtered = projects.value.filter(matchesSearch)

  filtered.forEach(project => {
    const groupKey = project.group || t('projects.ungrouped')
    if (!groups[groupKey]) {
      groups[groupKey] = []
    }
    groups[groupKey].push(project)
  })

  return groups
})

const filteredProjects = computed(() => {
  let result = projects.value.filter(project => {
    const matchesGroup = filterGroup.value === 'all' ||
      (filterGroup.value === 'ungrouped' && !project.group) ||
      project.group === filterGroup.value

    const matchesStatus = filterStatus.value === 'all' ||
      project.status === filterStatus.value

    return matchesSearch(project) && matchesGroup && matchesStatus
  })

  result.sort((a, b) => {
    let cmp = 0
    switch (sortBy.value) {
      case 'name':
        cmp = a.name.localeCompare(b.name)
        break
      case 'path':
        cmp = a.path.localeCompare(b.path)
        break
      case 'godotVersion':
        cmp = a.godot_version.localeCompare(b.godot_version)
        break
      case 'status':
        cmp = a.status.localeCompare(b.status)
        break
      case 'updatedAt':
        cmp = b.updated_at.localeCompare(a.updated_at)
        break
      default:
        cmp = a.name.localeCompare(b.name)
    }
    return sortOrder.value === 'asc' ? cmp : -cmp
  })

  return result
})

const {
  selectedIds: selectedProjectIds,
  isBatchMode,
  selectedCount,
  toggleSelection: toggleProjectSelection,
  selectAll: selectAllProjects,
  clearSelection,
} = useBatchSelection<Project>({
  items: filteredProjects,
  getId: (p) => p.project_id,
})

const loadGroups = async () => {
  try {
    availableGroups.value = await api.getProjectGroups()
  } catch (error) {
    console.error('Failed to load groups:', error)
  }
}

const showProjectDetails = async (project: Project, tab?: 'overview' | 'environment' | 'vcs') => {
  selectedProject.value = project
  showProjectDetail.value = true
  detailTab.value = tab || 'overview'
  showAddPluginPanel.value = false
  driftReport.value = null
  syncPreview.value = null
  syncResult.value = null
  try {
    projectBindings.value = await api.getProjectBindings(project.project_id)
    await loadPluginEnabledState()
    await checkDrift(project.project_id)
  } catch (error) {
    console.error('Failed to load project details:', error)
  }
}

const checkDrift = async (projectId: string) => {
  isCheckingDrift.value = true
  try {
    driftReport.value = await api.checkProjectDrift(projectId)
    driftMap.value.set(projectId, driftReport.value)
  } catch {
    driftReport.value = null
  } finally {
    isCheckingDrift.value = false
  }
}

const openSyncPreview = async () => {
  if (!selectedProject.value) return
  try {
    syncPreview.value = await api.previewSync(selectedProject.value.project_id)
    const allKeys = new Set(syncPreview.value.actions.map((a: any) => `${a.item_type}:${a.name}`))
    selectedSyncItems.value = allKeys
    showProjectDetail.value = false
    showSyncPreview.value = true
  } catch (e: any) {
    toast.error(`${t('projects.syncFailed')}: ${e?.toString() || e}`)
  }
}

const executeSync = async () => {
  if (!selectedProject.value) return
  isSyncing.value = true
  try {
    const onlyItems = selectedSyncItems.value.size > 0 ? Array.from(selectedSyncItems.value) : undefined
    syncResult.value = await api.syncProjectEnvironment(selectedProject.value.project_id, onlyItems)
    showSyncPreview.value = false
    if (syncResult.value.failed > 0) {
      toast.warning(t('projects.syncPartialWarning'))
    } else {
      toast.success(t('projects.syncEnvironmentSuccess'))
    }
    await checkDrift(selectedProject.value.project_id)
    projectBindings.value = await api.getProjectBindings(selectedProject.value.project_id)
  } catch (e: any) {
    toast.error(`${t('projects.syncFailed')}: ${e?.toString() || e}`)
  } finally {
    isSyncing.value = false
  }
}

const loadProjects = async (force = false) => {
  const hasData = projects.value.length > 0
  if (hasData && !force) {
    isRefreshing.value = true
    try {
      const result = await api.getProjects()
      projects.value = result
      loadError.value = null
      preloadIcons(result.map(p => p.icon_path).filter(Boolean)).catch(() => {})
      Promise.all([loadAllProjectBindings(), loadAllDrifts(), loadAllVcsInfo(), loadAllMissingModules(), loadAllLockStatuses()]).catch(() => {})
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
    const result = await api.getProjects()
    projects.value = result
    preloadIcons(result.map(p => p.icon_path).filter(Boolean)).catch(() => {})
    await Promise.all([loadGroups(), checkMovedProjects()])
    Promise.all([
      loadAllProjectBindings(),
      api.getPlugins().then(p => { allPlugins.value = p }).catch(() => { allPlugins.value = [] }),
      loadAllDrifts(),
      loadAllVcsInfo(),
      loadAllMissingModules(),
      loadAllLockStatuses(),
    ]).catch(() => {})
  } catch (error) {
    loadError.value = String(error)
  } finally {
    isLoading.value = false
  }
}

const loadAllDrifts = async () => {
  try {
    const reports = await api.checkAllDrifts()
    const map = new Map<string, DriftReport>()
    for (const report of reports) {
      map.set(report.project_id, report)
    }
    driftMap.value = map
  } catch {
    driftMap.value = new Map()
  }
}

const loadAllVcsInfo = async () => {
  try {
    const ids = projects.value.map(p => p.project_id)
    if (ids.length === 0) return
    const results = await api.batchGetVcsInfo(ids)
    const map = new Map<string, VcsInfo>()
    for (const [id, info] of results) {
      map.set(id, info)
    }
    vcsInfoMap.value = map
  } catch {
    vcsInfoMap.value = new Map()
  }
}

const loadAllMissingModules = async () => {
  try {
    const map = new Map<string, ModuleType[]>()
    const checks = projects.value.map(async (p) => {
      try {
        const missing = await api.checkProjectMissingModules(p.project_id)
        map.set(p.project_id, missing)
      } catch {
        // Silently ignore - this is informational
      }
    })
    await Promise.allSettled(checks)
    projectMissingModulesMap.value = map
  } catch {
    projectMissingModulesMap.value = new Map()
  }
}

const loadAllLockStatuses = async () => {
  try {
    const ids = projects.value.map(p => p.project_id)
    if (ids.length === 0) return
    const results = await api.batchCheckLocks(ids)
    const map = new Map<string, 'locked_verified' | 'locked_drifted' | 'not_locked'>()
    for (const [id, lock, verifyResult] of results) {
      if (!lock) {
        map.set(id, 'not_locked')
      } else if (verifyResult.is_valid) {
        map.set(id, 'locked_verified')
      } else {
        map.set(id, 'locked_drifted')
      }
    }
    lockStatusMap.value = map
  } catch {
    lockStatusMap.value = new Map()
  }
}

const loadAllProjectBindings = async () => {
  try {
    const map = await api.getAllProjectBindings()
    const bindingMap = new Map<string, ProjectBinding[]>()
    for (const [projectId, bindings] of Object.entries(map)) {
      bindingMap.set(projectId, bindings)
    }
    projectBindingMap.value = bindingMap
  } catch {
    projectBindingMap.value = new Map()
  }
}

const showMovedDialog = ref(false)
const movedCandidates = ref<MovedProjectCandidate[]>([])

const checkMovedProjects = async () => {
  try {
    const candidates = await api.detectMovedProjects()
    if (candidates.length > 0) {
      movedCandidates.value = candidates
      showMovedDialog.value = true
    }
  } catch (error) {
    console.error('检测迁移项目失败:', error)
  }
}

const confirmMovedProject = async (candidate: MovedProjectCandidate) => {
  try {
    await api.confirmProjectRelocation(candidate.project_id, candidate.new_path)
    toast.success(t('common.projectMigrated', { name: candidate.old_name }))
    movedCandidates.value = movedCandidates.value.filter(c => c.project_id !== candidate.project_id)
    if (movedCandidates.value.length === 0) {
      showMovedDialog.value = false
    }
    await loadProjects()
  } catch (error) {
    toast.error(t('common.migrationFailed', { error }))
  }
}

const dismissMovedProject = (candidate: MovedProjectCandidate) => {
  movedCandidates.value = movedCandidates.value.filter(c => c.project_id !== candidate.project_id)
  if (movedCandidates.value.length === 0) {
    showMovedDialog.value = false
  }
}

const selectScanDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.scanTitle')
    })
    if (selected && typeof selected === 'string') {
      scanDirInput.value = selected
    }
  } catch (error) {
    toast.error(t('common.selectDirFailed', { error }))
  }
}

const startScan = async () => {
  if (!scanDirInput.value) {
    toast.warning(t('common.selectDirFirst'))
    return
  }
  showScanDialog.value = false
  isLoading.value = true
  try {
    const result = await api.scanProjects([scanDirInput.value])
    projects.value = result
    toast.success(t('common.scanComplete', { count: result.length }))
    await loadProjects()
    if (result.length > 0) {
      runAutoSetup(result, true)
    }
  } catch (error) {
    toast.error(t('common.scanFailed', { error }))
  } finally {
    isLoading.value = false
    showScanDialog.value = false
  }
}

const quickScan = async () => {
  isLoading.value = true
  try {
    const settings = await api.getSettings()
    let rootDirs = settings.scan_directories
    if (rootDirs.length === 0) {
      rootDirs = await api.getDefaultScanDirs()
    }
    if (rootDirs.length === 0) {
      toast.warning(t('projects.noScanDirs'))
      showScanDialog.value = true
      isLoading.value = false
      return
    }
    const result = await api.scanProjects(rootDirs)
    projects.value = result
    toast.success(t('common.scanComplete', { count: result.length }))
    await loadProjects()
    if (result.length > 0) {
      runAutoSetup(result, true)
    }
  } catch (error) {
    toast.error(t('common.scanFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const quickScanFromDialog = async () => {
  showScanDialog.value = false
  await quickScan()
}

const addProject = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.scanTitle')
    })
    if (selected && typeof selected === 'string') {
      const existing = projects.value.find(p => p.path.replace(/\\/g, '/').toLowerCase() === selected.replace(/\\/g, '/').toLowerCase())
      if (existing) {
        toast.warning(t('projects.projectAlreadyExists', { name: existing.name }))
        return
      }
      isLoading.value = true
      const result = await api.addProject(selected)
      toast.success(t('common.addProjectSuccess', { name: result.name }))
      await loadProjects()
      runAutoSetup([result], true)
    }
  } catch (error) {
    toast.error(t('common.addProjectFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const importProjectFromGit = async () => {
  if (!gitUrl.value.trim()) {
    toast.warning(t('projects.gitImport.enterUrl'))
    return
  }
  isCloningFromGit.value = true
  try {
    const result = await api.importProjectFromGit(gitUrl.value.trim(), gitTargetDir.value.trim() || undefined)
    toast.success(t('projects.gitImport.success', { name: result.name }))
    showGitDialog.value = false
    showAddProjectDialog.value = false
    gitUrl.value = ''
    gitTargetDir.value = ''
    await loadProjects()
    runAutoSetup([result], true)
  } catch (error) {
    toast.error(t('projects.gitImport.failed', { error }))
  } finally {
    isCloningFromGit.value = false
  }
}

const browseGitTargetDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.gitImport.selectTargetDir')
    })
    if (selected && typeof selected === 'string') {
      gitTargetDir.value = selected
    }
  } catch { /* ignore */ }
}

const loadTemplatesForDialog = async () => {
  try {
    const [local, hub] = await Promise.all([
      api.listTemplates(),
      api.listHubTemplates()
    ])
    templateList.value = local
    hubTemplateList.value = hub
  } catch { /* ignore */ }
}

const openAddProjectDialog = async () => {
  showAddProjectDialog.value = true
  addProjectMode.value = 'local'
  loadTemplatesForDialog()
  try {
    const paths = await api.getStoragePaths()
    if (!gitTargetDir.value) gitTargetDir.value = paths.app_data_dir
    if (!templateTargetDir.value) templateTargetDir.value = paths.app_data_dir
  } catch { /* ignore */ }
}

const closeAddProjectDialog = () => {
  showAddProjectDialog.value = false
  gitUrl.value = ''
  gitTargetDir.value = ''
  selectedTemplateId.value = ''
  templateProjectName.value = ''
  templateTargetDir.value = ''
}

const createProjectFromTemplate = async () => {
  if (!selectedTemplateId.value || !templateProjectName.value.trim()) {
    toast.warning(t('projects.addProjectDialog.fillRequired'))
    return
  }
  isCreatingFromTemplate.value = true
  try {
    const rawId = selectedTemplateId.value.startsWith('hub:') ? selectedTemplateId.value.slice(4) : selectedTemplateId.value
    await api.instantiateTemplate(
      rawId,
      templateProjectName.value.trim(),
      templateTargetDir.value.trim() || '',
      false
    )
    toast.success(t('projects.addProjectDialog.templateCreated', { name: templateProjectName.value }))
    closeAddProjectDialog()
    await loadProjects()
  } catch (error) {
    toast.error(t('projects.addProjectDialog.templateFailed', { error }))
  } finally {
    isCreatingFromTemplate.value = false
  }
}

const browseTemplateTargetDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.addProjectDialog.selectTargetDir')
    })
    if (selected && typeof selected === 'string') {
      templateTargetDir.value = selected
    }
  } catch { /* ignore */ }
}

const showConfirmDialog = ref(false)
const confirmAction = ref<{ title: string; message: string; onConfirm: () => void } | null>(null)
const deleteWithFiles = ref(false)
const deletingProjectId = ref<string | null>(null)
const isBatchDeleting = ref(false)
const batchDeleteProgress = ref<{ current: number; total: number; message: string } | null>(null)

const showSingleDeleteConfirm = ref(false)
const singleDeleteTarget = ref<{ projectId: string; name: string } | null>(null)

const showGroupDeleteConfirm = ref(false)
const groupDeleteTarget = ref<{ groupId: string; name: string } | null>(null)

const onConfirmDialogConfirm = () => {
  if (confirmAction.value) {
    confirmAction.value.onConfirm()
  }
  showConfirmDialog.value = false
  confirmAction.value = null
}

const isDragging = ref(false)
const dragCounter = ref(0)

const onDragEnter = (e: DragEvent) => {
  e.preventDefault()
  dragCounter.value++
  isDragging.value = true
}

const onDragLeave = (e: DragEvent) => {
  e.preventDefault()
  dragCounter.value--
  if (dragCounter.value === 0) {
    isDragging.value = false
  }
}

const onDragOver = (e: DragEvent) => {
  e.preventDefault()
}

const onDrop = async (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false
  dragCounter.value = 0

  const files = e.dataTransfer?.files
  if (!files || files.length === 0) return

  const paths: string[] = []
  for (let i = 0; i < files.length; i++) {
    const path = (files[i] as any).path
    if (path && !projects.value.find(p => p.path.replace(/\\/g, '/').toLowerCase() === path.replace(/\\/g, '/').toLowerCase())) {
      paths.push(path)
    }
  }

  const duplicateCount = files.length - paths.length
  isLoading.value = true

  const results = await Promise.allSettled(
    paths.map(path => api.addProject(path))
  )

  isLoading.value = false
  await loadProjects()

  const addedCount = results.filter(r => r.status === 'fulfilled').length
  const skippedCount = results.filter(r => r.status === 'rejected').length

  if (duplicateCount > 0 && addedCount > 0) {
    toast.info(t('projects.dragDropResult', { added: addedCount, skipped: skippedCount + duplicateCount }))
  } else if (duplicateCount > 0 && addedCount === 0) {
    toast.warning(t('projects.projectAlreadyExists', { name: duplicateCount + '' }))
  } else if (addedCount > 0) {
    toast.success(t('projects.dragDropAdded', { count: addedCount }))
  }
}

const removeProject = async (projectId: string) => {
  const project = projects.value.find(p => p.project_id === projectId)
  const name = project?.name || projectId
  singleDeleteTarget.value = { projectId, name }
  deleteWithFiles.value = false
  showSingleDeleteConfirm.value = true
}

const onSingleDeleteConfirm = async () => {
  if (!singleDeleteTarget.value) return
  const { projectId } = singleDeleteTarget.value
  deletingProjectId.value = projectId
  try {
    const bindings = projectBindingMap.value.get(projectId) || []
    for (const binding of bindings) {
      try {
        await api.unbindPlugin(projectId, binding.plugin_id)
      } catch { /* ignore unbind errors during removal */ }
    }
    await api.removeProject(projectId, deleteWithFiles.value)
    selectedProjectIds.value.delete(projectId)
    selectedProjectIds.value = new Set(selectedProjectIds.value)
    if (selectedProjectIds.value.size === 0) {
      isBatchMode.value = false
    }
    toast.success(deleteWithFiles.value ? t('projects.deletedWithFiles') : t('common.projectDeleted'))
    await loadProjects()
  } catch (error) {
    toast.error(t('common.deleteFailed', { error }))
  } finally {
    deletingProjectId.value = null
    deleteWithFiles.value = false
    showSingleDeleteConfirm.value = false
    singleDeleteTarget.value = null
  }
}

const syncProject = async (project: Project) => {
  try {
    await api.syncProjects()
    await loadProjects()
    const synced = projects.value.find(p => p.project_id === project.project_id)
    if (synced) {
      toast.success(t('projects.syncSuccess', { name: project.name, status: t(`projects.status.${synced.status.toLowerCase()}`) }))
    }
  } catch (error) {
    toast.error(t('projects.syncFailed', { error }))
  }
}

const syncAllProjects = async () => {
  isLoading.value = true
  try {
    await api.syncProjects()
    await loadProjects()
    toast.success(t('projects.syncAllSuccess', { count: projects.value.length }))
  } catch (error) {
    toast.error(t('projects.syncFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const openGroupDialog = (project: Project) => {
  editingProjectId.value = project.project_id
  selectedGroupId.value = project.group || ''
  showGroupDialog.value = true
}

const saveGroup = async () => {
  if (!editingProjectId.value) return

  try {
    await api.updateProjectGroup(editingProjectId.value, selectedGroupId.value)
    const project = projects.value.find(p => p.project_id === editingProjectId.value)
    if (project) {
      project.group = selectedGroupId.value
    }
    toast.success(t('common.groupUpdated'))
    showGroupDialog.value = false
    editingProjectId.value = null
    selectedGroupId.value = ''
    await loadGroups()
  } catch (error) {
    toast.error(t('common.groupUpdateFailed', { error }))
  }
}

const loadEngines = async () => {
  try {
    engines.value = await api.getEngines()
  } catch (error) {
    console.error('Failed to load engines:', error)
  }
}

const openCreateGroupDialog = () => {
  newGroupName.value = ''
  newGroupIcon.value = ''
  newGroupColor.value = '#6B7280'
  newGroupDescription.value = ''
  showCreateGroupDialog.value = true
}

const createGroup = async () => {
  if (!newGroupName.value.trim()) {
    toast.warning(t('projects.groupNameRequired'))
    return
  }
  isCreatingGroup.value = true
  try {
    const group = await api.createProjectGroup(
      newGroupName.value.trim(),
      newGroupIcon.value || undefined,
      newGroupColor.value || undefined,
      newGroupDescription.value || undefined
    )
    await loadGroups()
    selectedGroupId.value = group.group_id
    showCreateGroupDialog.value = false
    toast.success(t('projects.groupCreated', { name: group.name }))
  } catch (error) {
    toast.error(String(error))
  } finally {
    isCreatingGroup.value = false
  }
}

const deleteGroup = async (groupId: string) => {
  const group = getGroupById(groupId)
  if (!group) return
  groupDeleteTarget.value = { groupId, name: group.name }
  showGroupDeleteConfirm.value = true
}

const onGroupDeleteConfirm = async () => {
  if (!groupDeleteTarget.value) return
  const { groupId, name } = groupDeleteTarget.value
  try {
    await api.deleteProjectGroup(groupId)
    if (selectedGroupId.value === groupId) {
      selectedGroupId.value = ''
    }
    await loadGroups()
    await loadProjects()
    toast.success(t('projects.groupDeleted', { name }))
  } catch (error) {
    toast.error(String(error))
  } finally {
    showGroupDeleteConfirm.value = false
    groupDeleteTarget.value = null
  }
}

const showRelocateDialog = ref(false)
const relocateProjectId = ref('')
const relocateNewPath = ref('')

const {
  showEngineSelectDialog,
  engineSelectProject,
  matchedEngines,
  isLoadingEngines,
  isLaunching,
  engineSelectMode,
  openProjectWithEngine,
  selectDefaultEngine,
  launchWithEngine,
  closeEngineSelectDialog,
  getMatchLevelClass,
  getMatchLevelLabel,
  getMatchLevelDesc,
} = useEngineLauncher(() => loadProjects())

const projectContextMenu = useContextMenu()

const showProjectContextMenu = (event: MouseEvent, project: Project) => {
  projectContextMenu.show(event, [
    {
      label: t('projects.contextMenu.openProject'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>',
      action: () => openProjectWithEngineWrapper(project),
      disabled: isLaunching,
    },
    { separator: true },
    {
      label: t('projects.contextMenu.editBindings'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>',
      action: () => goToPluginBindings(project),
    },
    {
      label: t('projects.contextMenu.applyChanges'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" /></svg>',
      action: async () => {
        try {
          const result = await api.applyChanges(project.project_id)
          if (!result.success) {
            toast.warning(t('linker.bindingApplyFailed', { errors: result.errors.join('; ') }))
          } else {
            toast.success(t('linker.pluginUnbound'))
          }
        } catch (error) {
          toast.error(String(error))
        }
      },
    },
    { separator: true },
    {
      label: t('projects.contextMenu.setGroup'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" /></svg>',
      action: () => openGroupDialog(project),
    },
  ] as ContextMenuEntry[])
}

const showProjectListContextMenu = (event: MouseEvent) => {
  projectContextMenu.show(event, [
    {
      label: t('projects.contextMenu.addProject'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 4v16m8-8H4" /></svg>',
      action: () => openAddProjectDialog(),
    },
    {
      label: t('projects.contextMenu.scanProjects'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>',
      action: () => quickScan(),
    },
    { separator: true },
    {
      label: t('projects.contextMenu.refresh'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>',
      action: () => loadProjects(true),
    },
  ] as ContextMenuEntry[])
}

useDialogEscape(showScanDialog)
useDialogEscape(showProjectDetail)
useDialogEscape(showGroupDialog)
useDialogEscape(showRelocateDialog)
useDialogEscape(showMovedDialog)
useDialogEscape(showBatchGroupDialog)
useDialogEscape(showGitDialog)
useDialogEscape(showAddProjectDialog)
useDialogEscape(showEngineSelectDialog)
useDialogEscape(showCreateGroupDialog)

const showSaveAsTemplateDialog = ref(false)
useDialogEscape(showSaveAsTemplateDialog)
const saveAsTemplateProjectId = ref('')
const saveAsTemplateName = ref('')
const saveAsTemplateCategory = ref('Custom')
const isSavingAsTemplate = ref(false)

const openSaveAsTemplateDialog = (project: Project) => {
  saveAsTemplateProjectId.value = project.project_id
  saveAsTemplateName.value = project.name
  saveAsTemplateCategory.value = 'Custom'
  showSaveAsTemplateDialog.value = true
}

const handleSaveAsTemplate = async () => {
  if (!saveAsTemplateName.value.trim()) return
  isSavingAsTemplate.value = true
  try {
    await api.generateTemplateFromProject(
      saveAsTemplateProjectId.value,
      saveAsTemplateName.value.trim(),
      saveAsTemplateCategory.value
    )
    toast.success(t('templates.saveSuccess'))
    showSaveAsTemplateDialog.value = false
  } catch (e: any) {
    toast.error(`${t('templates.generateFailed')}: ${e?.toString() || e}`)
  } finally {
    isSavingAsTemplate.value = false
  }
}

const openProjectWithEngineWrapper = async (project: Project) => {
  projectMenuId.value = ''
  await openProjectWithEngine(project, engines.value)
}

const openRelocateDialog = (project: Project) => {
  relocateProjectId.value = project.project_id
  relocateNewPath.value = ''
  showRelocateDialog.value = true
}

const selectRelocatePath = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.relocateTitle')
    })
    if (selected && typeof selected === 'string') {
      relocateNewPath.value = selected
    }
  } catch (error) {
    toast.error(t('common.selectDirFailed', { error }))
  }
}

const confirmRelocate = async () => {
  if (!relocateNewPath.value) {
    toast.warning(t('common.selectNewPath'))
    return
  }
  try {
    await api.relocateProject(relocateProjectId.value, relocateNewPath.value)
    toast.success(t('common.projectPathUpdated'))
    showRelocateDialog.value = false
    await loadProjects()
  } catch (error: any) {
    const msg = String(error)
    if (msg.includes('project.godot') || msg.includes('not a valid')) {
      toast.error(t('projects.invalidRelocatePath'))
    } else {
      toast.error(t('common.relocateFailed', { error }))
    }
  }
}

const goToPluginBindings = (project: Project) => {
  showProjectDetail.value = false
  showAddPluginPanel.value = false
  router.push({ path: '/plugins', query: { tab: 'bindings', project: project.project_id } })
}

const unbindProjectBinding = async (binding: ProjectBinding) => {
  try {
    try {
      await api.disablePluginInProject(binding.project_id, binding.plugin_id)
    } catch {
      // ignore disable failure
    }
    await api.unbindPlugin(binding.project_id, binding.plugin_id)
    const applyResult = await api.applyChanges(binding.project_id)
    if (!applyResult.success) {
      toast.warning(t('linker.bindingApplyFailed', { errors: applyResult.errors.join('; ') }))
    } else {
      toast.success(t('linker.pluginUnbound'))
    }
    if (selectedProject.value) {
      projectBindings.value = await api.getProjectBindings(selectedProject.value.project_id)
      projectBindingMap.value.set(selectedProject.value.project_id, projectBindings.value)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const repairProjectBinding = async (binding: ProjectBinding) => {
  try {
    await api.repairBinding(binding.project_id, binding.plugin_id)
    const applyResult = await api.applyChanges(binding.project_id)
    if (!applyResult.success) {
      toast.warning(t('linker.bindingApplyFailed', { errors: applyResult.errors.join('; ') }))
    } else {
      toast.success(t('linker.repairSuccess'))
    }
    if (selectedProject.value) {
      projectBindings.value = await api.getProjectBindings(selectedProject.value.project_id)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const pluginEnabledMap = ref<Map<string, boolean>>(new Map())

const loadPluginEnabledState = async () => {
  if (!selectedProject.value) return
  try {
    const enabledList = await api.getEnabledPlugins(selectedProject.value.project_id)
    const map = new Map<string, boolean>()
    for (const name of enabledList) {
      map.set(name, true)
    }
    pluginEnabledMap.value = map
  } catch {
    pluginEnabledMap.value = new Map()
  }
}

const isPluginEnabled = (binding: ProjectBinding): boolean => {
  const plugin = allPlugins.value.find(p => p.plugin_id === binding.plugin_id)
  if (!plugin) return false
  const unit = plugin.versions.flatMap(v => v.units).find(u => u.unit_id === binding.unit_id)
  if (!unit) return false
  const dirName = unit.subdirectory
    ? unit.subdirectory.replace(/\\/g, '/').split('/').pop() || unit.name
    : unit.name
  return pluginEnabledMap.value.get(dirName) ?? false
}

const togglePluginEnabled = async (binding: ProjectBinding) => {
  const enabled = isPluginEnabled(binding)
  try {
    if (enabled) {
      await api.disablePluginInProject(binding.project_id, binding.plugin_id)
      toast.success(t('plugins.pluginDisabled'))
    } else {
      await api.enablePluginInProject(binding.project_id, binding.plugin_id)
      toast.success(t('plugins.pluginEnabled'))
    }
    await loadPluginEnabledState()
  } catch (error) {
    toast.warning(t('plugins.enableDisableFailed', { error: String(error) }))
  }
}

const showAddPluginPanel = ref(false)
const addPluginSearchQuery = ref('')
const isBindingPlugin = ref(false)

const boundPluginIds = computed(() => new Set(projectBindings.value.map(b => b.plugin_id)))

const availablePluginsForProject = computed(() => {
  return allPlugins.value.filter(p => !boundPluginIds.value.has(p.plugin_id))
})

const filteredAvailablePlugins = computed(() => {
  const q = addPluginSearchQuery.value.toLowerCase().trim()
  if (!q) return availablePluginsForProject.value
  return availablePluginsForProject.value.filter(p =>
    p.name.toLowerCase().includes(q) ||
    p.description.toLowerCase().includes(q) ||
    p.author.toLowerCase().includes(q)
  )
})

const isCompatWarning = (plugin: Plugin, project: Project) => {
  if (plugin.compatibility === 'Both' || plugin.compatibility === 'Unknown') return false
  const projMajor = project.godot_version.split('.')[0]
  if (plugin.compatibility === 'Godot4' && projMajor !== '4') return true
  if (plugin.compatibility === 'Godot3' && projMajor !== '3') return true
  return false
}

const bindPluginInline = async (plugin: Plugin) => {
  if (!selectedProject.value || isBindingPlugin.value) return
  isBindingPlugin.value = true
  try {
    const version = plugin.versions[0]
    if (!version) return
    const unit = version.units[0]
    if (!unit) return
    const mountPath = `addons/${unit.name}`
    await api.bindPlugin(selectedProject.value.project_id, plugin.plugin_id, version.version_id, unit.unit_id, mountPath, unit.subdirectory || '')
    const applyResult = await api.applyChanges(selectedProject.value.project_id)
    if (!applyResult.success) {
      toast.warning(t('linker.bindingApplyFailed', { errors: applyResult.errors.join('; ') }))
    } else {
      try {
        await api.enablePluginInProject(selectedProject.value.project_id, plugin.plugin_id)
      } catch {
        // ignore enable failure - plugin is bound but not auto-enabled
      }
      toast.success(t('plugins.importPluginSuccess', { name: plugin.name }))
    }
    projectBindings.value = await api.getProjectBindings(selectedProject.value.project_id)
    projectBindingMap.value.set(selectedProject.value.project_id, projectBindings.value)
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isBindingPlugin.value = false
  }
}

const toggleAddPluginPanel = () => {
  showAddPluginPanel.value = !showAddPluginPanel.value
  if (showAddPluginPanel.value) {
    addPluginSearchQuery.value = ''
  }
}
</script>

<template>
  <div class="relative">
    <div v-if="isDragging" class="fixed inset-0 bg-primary-500/10 border-4 border-dashed border-primary-500 z-40 flex items-center justify-center pointer-events-none">
      <div class="dialog-container max-w-md w-full mx-4">
        <svg class="mx-auto h-12 w-12 text-primary-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <p class="text-sm font-semibold text-primary-600 dark:text-brand-primary">{{ t('projects.dragTitle') }}</p>
        <p class="text-sm text-gray-500 dark:text-content-muted mt-1">{{ t('projects.dragDesc') }}</p>
      </div>
    </div>
    <div
      class="space-y-0"
      @dragenter="onDragEnter"
      @dragleave="onDragLeave"
      @dragover="onDragOver"
      @drop="onDrop"
    >
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2 pb-2">
      <h1 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('projects.title') }}</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="quickScan"
          :disabled="isLoading"
          class="btn-primary disabled:opacity-50 text-sm"
        >
          {{ t('projects.scan') }}
        </button>
        <button
          @click="openAddProjectDialog"
          :disabled="isLoading"
          class="btn-secondary disabled:opacity-50 text-sm flex items-center gap-1.5"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 4v16m8-8H4" /></svg>
          {{ t('projects.addProject') }}
        </button>
        <button
          @click="syncAllProjects"
          :disabled="isLoading"
          class="btn-secondary disabled:opacity-50 text-sm flex items-center gap-1.5"
          :title="t('projects.syncAllHint')"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ t('projects.syncProject') }}
        </button>
      </div>
    </div>

    <div class="border-b border-gray-200/60 dark:border-surface-border/40 pb-2">
      <div class="flex flex-col lg:flex-row gap-2">
        <div class="flex-1">
          <input
            ref="searchInputRef"
            v-model="searchQuery"
            type="text"
            :placeholder="t('projects.search')"
            class="input-field"
          />
        </div>
        <div class="flex flex-wrap gap-1.5 items-center">
          <select
            v-model="filterGroup"
            class="select-field"
          >
            <option value="all">{{ t('projects.allGroups') }}</option>
            <option value="ungrouped">{{ t('projects.ungrouped') }}</option>
            <option v-for="group in availableGroups" :key="group.group_id" :value="group.group_id">{{ group.icon }} {{ group.name }}</option>
          </select>
          <select
            v-model="filterStatus"
            class="select-field"
          >
            <option value="all">{{ t('projects.allStatus') }}</option>
            <option value="Ready">{{ t('projects.status.ready') }}</option>
            <option value="Warning">{{ t('projects.status.warning') }}</option>
            <option value="Error">{{ t('projects.status.error') }}</option>
            <option value="Conflict">{{ t('projects.status.conflict') }}</option>
            <option value="MissingSource">{{ t('projects.status.missingSource') }}</option>
          </select>
          <select
            v-model="sortBy"
            class="select-field"
          >
            <option value="name">{{ t('projects.sortByName') }}</option>
            <option value="path">{{ t('projects.sortByPath') }}</option>
            <option value="godotVersion">{{ t('projects.sortByVersion') }}</option>
            <option value="status">{{ t('projects.sortByStatus') }}</option>
            <option value="updatedAt">{{ t('projects.sortByUpdated') }}</option>
          </select>
          <button
            @click="sortOrder = sortOrder === 'asc' ? 'desc' : 'asc'"
            class="filter-btn"
            :title="sortOrder === 'asc' ? t('projects.ascending') : t('projects.descending')"
          >
            <svg v-if="sortOrder === 'asc'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 15l7-7 7 7" />
            </svg>
            <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div v-if="isBatchMode && selectedCount > 0" class="bg-primary-50 dark:bg-surface-hover border border-primary-200 dark:border-surface-border rounded-[6px] p-2 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <span class="text-sm font-medium text-primary-700 dark:text-content-secondary">{{ t('projects.selectedCount', { count: selectedCount }) }}</span>
        <button
          @click="selectAllProjects"
          class="text-xs text-primary-600 dark:text-brand-primary hover:underline"
        >
          {{ t('common.selectAll') }}
        </button>
        <button
          @click="clearSelection"
          class="text-xs text-gray-500 dark:text-content-muted hover:underline"
        >
          {{ t('common.deselectAll') }}
        </button>
      </div>
      <div class="flex gap-2">
        <button
          @click="batchSetGroup"
          class="px-3 py-1.5 btn-primary text-sm transition-colors"
        >
          {{ t('projects.batchSetGroup') }} ({{ selectedCount }})
        </button>
        <button
          @click="batchApplyChanges"
          :disabled="isBatchApplying"
          class="px-3 py-1.5 bg-green-600 text-white text-sm rounded-btn hover:bg-green-700 transition-colors disabled:opacity-50"
        >
          {{ isBatchApplying ? t('common.loading') : t('projects.batchApplyChanges') }} ({{ selectedCount }})
        </button>
        <button
          @click="batchRemoveProjects"
          :disabled="isBatchDeleting"
          class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-btn hover:bg-red-700 transition-colors disabled:opacity-50 flex items-center gap-1.5"
        >
          <svg v-if="isBatchDeleting" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" /></svg>
          {{ isBatchDeleting ? t('common.deleting') : `${t('common.batchDelete')} (${selectedCount})` }}
        </button>
      </div>
    </div>

    <div v-if="isLoading" class="py-3">
      <SkeletonList :count="4" type="project" />
    </div>

    <ErrorState
      v-else-if="loadError"
      :title="t('common.loadFailed', { error: '' })"
      :description="loadError"
      :retryLabel="t('common.retry')"
      @retry="loadProjects"
    />

    <div v-else-if="isAutoSetupRunning && filteredProjects.length === 0" class="text-center py-16">
      <div class="animate-spin rounded-full h-10 w-10 border-2 border-primary-600 border-t-transparent mx-auto"></div>
      <h3 class="mt-4 text-sm font-medium text-gray-900 dark:text-content-primary">{{ autoSetupMessage }}</h3>
      <p class="mt-1 text-xs text-gray-500 dark:text-content-muted">{{ t('autoSetup.pleaseWait') }}</p>
    </div>

    <EmptyState
      v-else-if="filteredProjects.length === 0"
      :title="t('projects.empty')"
      :description="t('projects.emptyDesc')"
      :actionLabel="t('projects.scan')"
      @action="showScanDialog = true"
      :shortcuts="[
        { key: 'Ctrl+K', description: t('commandPalette.title') },
      ]"
    />

    <div v-else class="space-y-0" @contextmenu="showProjectListContextMenu($event)">
      <div v-for="(groupProjects, groupName) in (filterGroup === 'all' ? groupedProjects : { all: filteredProjects })" :key="groupName" class="space-y-0">
        <div v-if="filterGroup === 'all' && Object.keys(groupedProjects).length > 1" class="flex items-center gap-2 px-3 py-1.5 bg-gray-50/80 dark:bg-surface-hover/30 border-l-2 border-l-primary-500 dark:border-l-brand-primary border-b border-gray-200/60 dark:border-surface-border/50">
          <span
            v-if="getGroupIcon(groupName) || groupName !== t('projects.ungrouped')"
            class="inline-flex items-center gap-1"
          >
            <span v-if="getGroupIcon(groupName)" class="text-xs">{{ getGroupIcon(groupName) }}</span>
            <span
              v-if="groupName !== t('projects.ungrouped')"
              class="w-2 h-2 rounded-full"
              :style="{ backgroundColor: getGroupColor(groupName) }"
            ></span>
          </span>
          <h2 class="section-header !px-0 !py-0">
            {{ getGroupName(groupName) }}
          </h2>
          <span class="text-xs text-gray-400 dark:text-content-muted">({{ groupProjects.length }})</span>
        </div>
        <div class="space-y-0">
          <div
            v-for="project in (filterGroup === 'all' ? groupProjects : filteredProjects)"
            :key="project.project_id"
            :class="[
              'px-3 py-1.5 border-b border-gray-200 dark:border-surface-border/40 flex items-center gap-3 hover:bg-black/[0.03] dark:hover:bg-white/[0.04] transition-colors',
              selectedProjectIds.has(project.project_id) ? 'bg-primary-50/50 dark:bg-primary-900/10' : ''
            ]"
            @contextmenu="showProjectContextMenu($event, project)"
            >
            <input
              type="checkbox"
              :checked="selectedProjectIds.has(project.project_id)"
              @click.stop="toggleProjectSelection(project, $event)"
              class="checkbox-field"
            />
            <div class="w-7 h-7 rounded-[4px] overflow-hidden bg-gray-100 dark:bg-surface-hover flex items-center justify-center flex-shrink-0">
              <img
                v-if="project.icon_path && getIconUrl(project.icon_path)"
                :src="getIconUrl(project.icon_path)"
                :alt="project.name"
                class="w-7 h-7 object-contain"
              />
              <svg v-else class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <div v-if="debugMode && project.icon_path" class="text-[9px] text-red-500 break-all leading-tight max-w-[200px]">
              <div>path: {{ project.icon_path }}</div>
              <div class="text-blue-500">{{ getIconDebugInfo(project.icon_path) }}</div>
            </div>
            <div 
              class="min-w-0 flex-1 cursor-pointer hover:text-primary-600 dark:hover:text-brand-primary"
              @click="showProjectDetails(project)"
            >
              <div class="flex items-center gap-2">
                <h3 class="text-sm font-medium text-gray-900 dark:text-content-primary">
                  {{ project.name }}
                </h3>
                <span
                  v-if="project.group"
                  @click.stop="openGroupDialog(project)"
                  class="inline-flex items-center gap-1 px-1.5 py-0.5 text-xs rounded-full font-medium cursor-pointer hover:opacity-80"
                  :style="{ backgroundColor: getGroupColor(project.group) + '20', color: getGroupColor(project.group) }"
                >
                  <span v-if="getGroupIcon(project.group)">{{ getGroupIcon(project.group) }}</span>
                  {{ getGroupName(project.group) }}
                </span>
                <span
                  v-else
                  @click.stop="openGroupDialog(project)"
                  class="text-gray-400 hover:text-primary-600 dark:hover:text-brand-primary p-0.5 rounded-[4px] hover:bg-gray-100 dark:hover:bg-surface-hover"
                  :title="t('projects.setGroup')"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
                  </svg>
                </span>
                <span
                  :class="getStatusBadgeClass(project.status)"
                >
                  {{ t(`projects.status.${project.status.toLowerCase()}`) }}
                </span>
                <VcsStatusBadge
                  v-if="vcsInfoMap.get(project.project_id) && vcsInfoMap.get(project.project_id)!.vcs_type === 'Git' && vcsInfoMap.get(project.project_id)!.status !== 'Clean'"
                  :vcsInfo="vcsInfoMap.get(project.project_id) || null"
                  @click="showProjectDetails(project)"
                />
              </div>
              <div class="flex items-center gap-2 mt-0.5">
                <span class="text-[11px] text-gray-500 dark:text-content-secondary truncate" :title="project.path">
                  {{ project.path }}
                </span>
                <span class="text-[11px] text-gray-300 dark:text-content-muted">·</span>
                <span class="text-[11px] text-gray-500 dark:text-content-secondary">Godot {{ project.godot_version }}</span>
                <span
                  v-if="projectBindingMap.get(project.project_id)?.some(b => b.is_healthy === false)"
                  class="text-sm text-red-500 flex items-center gap-1 cursor-pointer hover:text-red-600 transition-colors"
                  :title="t('projects.unhealthyBindings')"
                  @click.stop="showProjectDetails(project)"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                  {{ t('projects.unhealthyBindings') }}
                </span>
                <span
                  v-if="driftMap.get(project.project_id)?.has_drift || lockStatusMap.get(project.project_id) === 'locked_drifted'"
                  class="text-sm text-amber-500 flex items-center gap-1 cursor-pointer hover:text-amber-600 transition-colors"
                  :title="t('projects.environmentChanged')"
                  @click.stop="showProjectDetails(project, 'environment')"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                  {{ t('projects.environmentChanged') }}
                </span>
              </div>
            </div>
            <div class="flex items-center gap-1">
              <button
                @click.stop="openProjectWithEngineWrapper(project)"
                :disabled="isLaunching"
                class="p-2.5 rounded-[4px] text-gray-500 dark:text-content-muted hover:text-primary-600 dark:hover:text-brand-primary hover:bg-primary-50 dark:hover:bg-surface-hover transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                :title="t('projects.openWithEngine')"
              >
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
              </button>
              <button
                v-if="project.status === 'MissingSource'"
                @click.stop="openRelocateDialog(project)"
                class="px-3 py-1.5 rounded-btn text-sm font-medium bg-orange-500 text-white hover:bg-orange-600 transition-colors flex items-center gap-1.5"
                :title="t('projects.relocate')"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                {{ t('projects.relocate') }}
              </button>
              <div class="project-menu-wrapper" style="position: relative; display: inline-block">
                <button
                  @click.stop="toggleProjectMenu(project.project_id)"
                  class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 p-2 rounded-[4px] hover:bg-gray-100 dark:hover:bg-surface-layer transition-colors"
                  :title="t('projects.moreActions')"
                >
                  <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                  </svg>
                </button>
                <div
                  v-if="projectMenuId === project.project_id"
                  class="absolute right-0 top-full mt-1 bg-white dark:bg-surface-hover rounded-[6px] border border-gray-200/60 dark:border-surface-border/40 py-1 z-20 whitespace-nowrap"
                >
                  <button
                    @click.stop="showProjectDetails(project); showAddPluginPanel = true; projectMenuId = ''"
                    class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 4v16m8-8H4" /></svg>
                    {{ t('linker.bindPlugins') }}
                  </button>
                  <button
                    @click.stop="syncProject(project); projectMenuId = ''"
                    class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
                    {{ t('projects.syncProject') }}
                  </button>
                  <button
                    @click.stop="openInFileManager(project.path); projectMenuId = ''"
                    class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
                    {{ t('projects.openInFileManager') }}
                  </button>
                  <button
                    @click.stop="selectDefaultEngine(project); projectMenuId = ''"
                    class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
                    {{ t('projects.selectDefaultEngine') }}
                  </button>
                  <button
                    @click.stop="openSaveAsTemplateDialog(project); projectMenuId = ''"
                    class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" /></svg>
                    {{ t('templates.saveFromProject') }}
                  </button>
                  <hr class="my-1 border-gray-200/60 dark:border-surface-border/40" />
                  <button
                    @click.stop="removeProject(project.project_id); projectMenuId = ''"
                    :disabled="deletingProjectId === project.project_id"
                    class="w-full text-left px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2 disabled:opacity-50"
                  >
                    <svg v-if="deletingProjectId === project.project_id" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" /></svg>
                    <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                    {{ deletingProjectId === project.project_id ? t('common.deleting') : t('projects.delete') }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    </div>
  </div>

  <Teleport to="body">
  <div v-if="showProjectDetail && selectedProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showProjectDetail = false; selectedProject = null; showAddPluginPanel = false">
      <div class="dialog-container w-full max-w-2xl max-h-[90vh] overflow-y-auto" @click.stop>
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="flex-shrink-0 w-12 h-12 rounded-[6px] overflow-hidden bg-gray-100 dark:bg-surface-hover flex items-center justify-center">
              <img
                v-if="selectedProject.icon_path && getIconUrl(selectedProject.icon_path)"
                :src="getIconUrl(selectedProject.icon_path)"
                :alt="selectedProject.name"
                class="w-12 h-12 object-contain"
              />
              <svg v-else class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <div>
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary">
                {{ selectedProject.name }}
              </h3>
              <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-content-muted">
                <span>Godot {{ selectedProject.godot_version }}</span>
                <span>·</span>
                <span
                  :class="['px-2 py-0.5 rounded-[4px] text-xs font-medium', getStatusInlineClass(selectedProject.status)]"
                >
                  {{ t(`projects.status.${selectedProject.status.toLowerCase()}`) }}
                </span>
                <span v-if="selectedProject.last_synced_at" class="text-xs text-gray-400 dark:text-content-muted">
                  {{ t('projects.lastSynced') }} {{ new Date(selectedProject.last_synced_at).toLocaleString() }}
                </span>
              </div>
            </div>
          </div>
          <button
            @click="showProjectDetail = false; selectedProject = null; showAddPluginPanel = false"
            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="flex items-center gap-2 mb-4 text-sm text-gray-500 dark:text-content-muted bg-gray-50 dark:bg-surface-hover rounded-[6px] p-2.5">
          <span class="break-all flex-1">{{ selectedProject.path }}</span>
          <button
            @click="openInFileManager(selectedProject.path)"
            class="text-primary-600 hover:text-primary-800 dark:text-brand-primary p-1 flex-shrink-0"
            :title="t('projects.openInFileManager')"
          >
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </button>
        </div>

        <div v-if="selectedProject.status === 'MissingSource'" class="mb-4 p-3 bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-[6px]">
          <div class="flex items-start gap-2">
            <svg class="w-5 h-5 text-orange-500 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
            <div class="flex-1">
              <p class="text-sm font-medium text-orange-800 dark:text-orange-300">{{ t('projects.status.missingSource') }}</p>
              <p class="text-xs text-orange-600 dark:text-orange-400 mt-1">{{ t('projects.relocateDesc') }}</p>
              <button
                @click="showProjectDetail = false; openRelocateDialog(selectedProject!)"
                class="mt-2 px-3 py-1.5 bg-orange-500 text-white text-xs rounded-btn hover:bg-orange-600 transition-colors"
              >
                {{ t('projects.relocate') }}
              </button>
            </div>
          </div>
        </div>

        <div class="flex border-b border-gray-200/60 dark:border-surface-border/40 mb-4">
          <button
            @click="detailTab = 'overview'"
            :class="['px-4 py-2 text-sm font-medium border-b-2 transition-colors', detailTab === 'overview' ? 'border-primary-600 text-primary-600 dark:border-brand-primary dark:text-brand-primary' : 'border-transparent text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-content-secondary']"
          >
            {{ t('projects.tabOverview') }}
          </button>
          <button
            @click="detailTab = 'environment'"
            :class="['px-4 py-2 text-sm font-medium border-b-2 transition-colors', detailTab === 'environment' ? 'border-primary-600 text-primary-600 dark:border-brand-primary dark:text-brand-primary' : 'border-transparent text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-content-secondary']"
          >
            {{ t('projects.tabEnvironment') }}
          </button>
          <button
            @click="detailTab = 'vcs'"
            :class="['px-4 py-2 text-sm font-medium border-b-2 transition-colors', detailTab === 'vcs' ? 'border-primary-600 text-primary-600 dark:border-brand-primary dark:text-brand-primary' : 'border-transparent text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-content-secondary']"
          >
            {{ t('projects.tabVcs') }}
          </button>
        </div>

        <div v-if="detailTab === 'overview'">
          <MissingModulesWarning :projectId="selectedProject.project_id" @install="handleInstallModule" />

          <div class="mb-4">
            <div class="flex items-center justify-between mb-2">
              <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('projects.pluginBindings') }}</h4>
              <button
                @click="goToPluginBindings(selectedProject!)"
                class="px-2.5 py-1 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary text-xs rounded-btn hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors flex items-center gap-1"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
                {{ t('linker.goToPluginEcosystem') }}
              </button>
            </div>
            <p class="text-xs text-gray-400 dark:text-content-muted mb-2">{{ t('plugins.pluginReloadHint') }}</p>

            <div v-if="projectBindings.length === 0" class="text-sm text-gray-500 dark:text-content-muted mb-3">
              {{ t('projects.noBindings') }}
            </div>
            <div v-else class="space-y-2 max-h-48 overflow-y-auto mb-3">
              <div
                v-for="binding in projectBindings"
                :key="binding.plugin_id + binding.mount_path"
                class="flex items-center justify-between p-2 rounded-[4px]"
                :class="binding.is_healthy === false ? 'bg-red-50 dark:bg-red-900/10 border border-red-200 dark:border-red-800' : 'bg-gray-50 dark:bg-surface-hover'"
              >
                <div class="flex items-center gap-2 min-w-0 flex-1">
                  <span v-if="binding.is_healthy === false" class="flex-shrink-0">
                    <svg class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                    </svg>
                  </span>
                  <span v-else class="flex-shrink-0">
                    <svg class="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
                    </svg>
                  </span>
                  <div class="min-w-0 flex-1">
                    <span class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ getPluginName(binding.plugin_id) }}</span>
                    <span class="text-xs text-gray-400 dark:text-content-muted ml-1.5">{{ getPluginVersion(binding.plugin_id) }}</span>
                    <span class="text-xs text-gray-500 dark:text-content-muted ml-2 font-mono">{{ binding.mount_path }}</span>
                  </div>
                </div>
                <div class="flex items-center gap-1 ml-2 flex-shrink-0">
                  <button
                    @click="togglePluginEnabled(binding)"
                    :class="['px-2 py-1 text-xs rounded-[4px]', isPluginEnabled(binding) ? 'text-green-600 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20' : 'text-gray-500 dark:text-content-muted hover:bg-gray-50 dark:hover:bg-surface-layer']"
                  >
                    {{ isPluginEnabled(binding) ? t('plugins.pluginEnabled') : t('plugins.pluginEnable') }}
                  </button>
                  <button
                    v-if="binding.is_healthy === false"
                    @click="repairProjectBinding(binding)"
                    class="px-2 py-1 text-xs text-orange-600 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/20 rounded-[4px]"
                  >
                    {{ t('plugins.bindDialog.repair') }}
                  </button>
                  <button
                    @click="unbindProjectBinding(binding)"
                    class="px-2 py-1 text-xs text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-[4px]"
                  >
                    {{ t('linker.unbind') }}
                  </button>
                </div>
              </div>
            </div>

            <div class="border-t border-gray-200/60 dark:border-surface-border/40 pt-3">
              <button
                @click="toggleAddPluginPanel"
                :class="['w-full px-3 py-2 text-sm rounded-btn transition-all flex items-center justify-center gap-2', showAddPluginPanel ? 'bg-gray-100 dark:bg-surface-hover text-gray-600 dark:text-content-secondary border border-gray-300 dark:border-surface-border' : 'btn-primary']"
              >
                <svg v-if="!showAddPluginPanel" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 4v16m8-8H4" />
                </svg>
                <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
                </svg>
                {{ showAddPluginPanel ? t('linker.cancelBind') : t('linker.bindPlugins') }}
              </button>
            </div>

            <div v-if="showAddPluginPanel" class="mt-2 border border-primary-200 dark:border-surface-border rounded-[6px] overflow-hidden">
              <div class="p-2 bg-primary-50 dark:bg-surface-hover border-b border-primary-200 dark:border-surface-border">
                <input
                  v-model="addPluginSearchQuery"
                  type="text"
                  :placeholder="t('plugins.search')"
                  class="w-full input-field text-xs"
                />
              </div>
              <div class="max-h-48 overflow-y-auto">
                <div v-if="availablePluginsForProject.length === 0" class="p-4 text-center text-xs text-gray-500 dark:text-content-muted">
                  {{ t('plugins.empty') }}
                </div>
                <div v-else-if="filteredAvailablePlugins.length === 0" class="p-4 text-center text-xs text-gray-500 dark:text-content-muted">
                  {{ t('plugins.searchNoResult') }}
                </div>
                <div
                  v-for="plugin in filteredAvailablePlugins"
                  :key="plugin.plugin_id"
                  class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-surface-border/40 last:border-0 hover:bg-gray-50 dark:hover:bg-surface-layer"
                >
                  <div class="min-w-0 flex-1">
                    <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1">
                      {{ plugin.name }}
                      <span v-if="isCompatWarning(plugin, selectedProject!)" class="text-xs text-orange-500" :title="t('plugins.bindDialog.compatWarning')">⚠</span>
                    </div>
                    <div class="text-xs text-gray-500 dark:text-content-secondary">v{{ plugin.versions[0]?.version || '1.0.0' }} · {{ plugin.author || t('plugins.unknownAuthor') }}</div>
                  </div>
                  <button
                    @click="bindPluginInline(plugin)"
                    :disabled="isBindingPlugin"
                    class="px-2 py-1 btn-primary text-xs ml-2 flex-shrink-0 disabled:opacity-50"
                  >
                    {{ isBindingPlugin ? t('common.loading') : t('linker.bind') }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div
            v-if="driftMap.get(selectedProject.project_id)?.has_drift || lockStatusMap.get(selectedProject.project_id) === 'locked_drifted'"
            class="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-[6px] flex items-center justify-between"
          >
            <div class="flex items-center gap-2">
              <svg class="w-4 h-4 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
              <span class="text-sm text-amber-700 dark:text-amber-400">{{ t('projects.environmentChanged') }}</span>
            </div>
            <button
              @click="detailTab = 'environment'"
              class="text-xs text-primary-600 hover:text-primary-800 dark:text-brand-primary font-medium"
            >
              {{ t('projects.viewEnvironment') }} &rarr;
            </button>
          </div>
        </div>

        <div v-if="detailTab === 'environment'">
          <div class="mb-4">
            <div class="flex items-center justify-between mb-2">
              <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('projects.environmentStatus') }}</h4>
              <button
                @click="checkDrift(selectedProject!.project_id)"
                :disabled="isCheckingDrift"
                class="text-xs text-primary-600 hover:text-primary-800 dark:text-brand-primary disabled:opacity-50"
              >
                {{ isCheckingDrift ? '...' : t('projects.recheck') }}
              </button>
            </div>
            <div v-if="isCheckingDrift" class="text-sm text-gray-400">{{ t('common.loading') }}</div>
            <div v-else-if="driftReport">
              <div v-if="!driftReport.has_drift" class="flex items-center gap-2 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-[6px]">
                <svg class="w-5 h-5 text-green-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
                </svg>
                <span class="text-sm text-green-700 dark:text-green-400">{{ t('projects.environmentInSync') }}</span>
              </div>
              <div v-else class="space-y-2">
                <div v-for="item in driftReport.items" :key="`${item.item_type}-${item.name}`"
                  class="flex items-start gap-2 p-2.5 rounded-[4px] text-sm"
                  :class="{
                    'bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800': item.status === 'VersionMismatch',
                    'bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800': item.status === 'Missing',
                    'bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800': item.status === 'Unexpected',
                  }"
                >
                  <svg v-if="item.status === 'VersionMismatch'" class="w-4 h-4 text-yellow-500 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                  <svg v-else-if="item.status === 'Missing'" class="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                  <svg v-else class="w-4 h-4 text-orange-500 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <span :class="{
                    'text-yellow-700 dark:text-yellow-400': item.status === 'VersionMismatch',
                    'text-red-700 dark:text-red-400': item.status === 'Missing',
                    'text-orange-700 dark:text-orange-400': item.status === 'Unexpected',
                  }">{{ item.message }}</span>
                </div>
                <button
                  @click="openSyncPreview"
                  class="w-full mt-2 py-2 text-sm font-medium btn-primary transition-colors"
                >
                  {{ t('projects.syncEnvironment') }}
                </button>
              </div>
            </div>
            <div v-else class="text-sm text-gray-400">{{ t('projects.noDriftData') }}</div>
          </div>

          <div class="mb-4">
            <details>
              <summary class="flex items-center justify-between cursor-pointer py-2 text-sm font-medium text-gray-700 dark:text-content-secondary hover:text-primary-600 dark:hover:text-brand-primary">
                <span>{{ t('projects.lockfileSection') }}</span>
                <svg class="w-4 h-4 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 9l-7 7-7-7" />
                </svg>
              </summary>
              <LockfilePanel :projectId="selectedProject.project_id" />
            </details>
          </div>

          <div class="mb-4">
            <details>
              <summary class="flex items-center justify-between cursor-pointer py-2 text-sm font-medium text-gray-700 dark:text-content-secondary hover:text-primary-600 dark:hover:text-brand-primary">
                <span>{{ t('projects.snapshotSection') }}</span>
                <svg class="w-4 h-4 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 9l-7 7-7-7" />
                </svg>
              </summary>
              <EnvironmentSnapshotPanel :projectId="selectedProject.project_id" />
            </details>
          </div>
        </div>

        <div v-if="detailTab === 'vcs'">
          <VcsPanel :projectId="selectedProject.project_id" />
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Sync Preview Dialog -->
  <Teleport to="body">
    <div v-if="showSyncPreview && syncPreview" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showSyncPreview = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('projects.syncPreviewTitle') }}</h3>
        <div v-if="syncPreview.actions.length === 0" class="text-sm text-gray-500 dark:text-content-muted mb-3">
          {{ t('projects.noActionsNeeded') }}
        </div>
        <div v-else class="space-y-2 mb-4 max-h-60 overflow-y-auto">
          <label
            v-for="(action, idx) in syncPreview.actions"
            :key="idx"
            class="flex items-center gap-2 p-2 rounded-[4px] text-sm cursor-pointer"
            :class="{
              'bg-blue-50 dark:bg-surface-hover': action.action_type === 'install',
              'bg-yellow-50 dark:bg-yellow-900/20': action.action_type === 'update',
              'bg-orange-50 dark:bg-orange-900/20': action.action_type === 'remove',
              'opacity-50': !selectedSyncItems.has(`${action.item_type}:${action.name}`),
            }"
          >
            <input
              type="checkbox"
              :checked="selectedSyncItems.has(`${action.item_type}:${action.name}`)"
              @change="(e: any) => {
                const key = `${action.item_type}:${action.name}`
                const s = new Set(selectedSyncItems)
                if (e.target.checked) s.add(key); else s.delete(key)
                selectedSyncItems = s
              }"
              class="checkbox-field"
            />
            <span class="font-medium px-1.5 py-0.5 rounded-[4px] text-xs"
              :class="{
                'bg-blue-100 text-blue-700 dark:bg-surface-border dark:text-content-secondary': action.action_type === 'install',
                'bg-yellow-100 text-yellow-700 dark:bg-yellow-800 dark:text-yellow-300': action.action_type === 'update',
                'bg-orange-100 text-orange-700 dark:bg-orange-800 dark:text-orange-300': action.action_type === 'remove',
              }"
            >
              {{ action.action_type === 'install' ? t('projects.actionInstall') : action.action_type === 'update' ? t('projects.actionUpdate') : t('projects.actionRemove') }}
            </span>
            <span class="text-gray-700 dark:text-content-secondary">{{ action.detail }}</span>
          </label>
        </div>
        <div v-if="syncResult" class="mb-4 p-3 rounded-[6px] text-sm"
          :class="syncResult.failed > 0 ? 'bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800' : 'bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800'"
        >
          <p class="font-medium mb-1">{{ t('projects.syncResult') }}</p>
          <p>{{ t('projects.syncedCount') }}: {{ syncResult.synced }} | {{ t('projects.skippedCount') }}: {{ syncResult.skipped }} | {{ t('projects.failedCount') }}: {{ syncResult.failed }}</p>
          <div v-if="syncResult.details.length > 0" class="mt-1 text-xs space-y-0.5">
            <p v-for="(d, i) in syncResult.details" :key="i">{{ d }}</p>
          </div>
        </div>
        <div class="flex gap-3">
          <button @click="showSyncPreview = false" class="flex-1 py-2.5 text-sm font-medium rounded-btn border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors">
            {{ t('common.cancel') }}
          </button>
          <button
            @click="executeSync"
            :disabled="isSyncing || selectedSyncItems.size === 0"
            class="flex-1 py-2.5 text-sm font-medium btn-primary transition-colors disabled:opacity-50"
          >
            {{ isSyncing ? '...' : t('projects.confirmSync') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showScanDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showScanDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('projects.scanTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-3">
          {{ t('projects.scanDesc') }}
        </p>
        <button
          v-if="hasScanDirs"
          @click="quickScanFromDialog"
          class="w-full mb-4 px-4 py-2.5 border border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover text-primary-700 dark:text-content-secondary rounded-btn hover:bg-primary-100 dark:hover:bg-surface-hover text-sm text-left flex items-center gap-2"
        >
          <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          <span>{{ t('projects.quickScanHint') }}</span>
        </button>
        <div class="flex gap-2 mb-6">
          <input
            v-model="scanDirInput"
            type="text"
            :placeholder="t('projects.scanPlaceholder')"
            class="flex-1 input-field"
          />
          <button
              @click="selectScanDir"
              class="btn-secondary text-sm whitespace-nowrap"
            >
              {{ t('projects.browse') }}
            </button>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            @click="showScanDialog = false"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="startScan"
            :disabled="!scanDirInput"
            class="btn-primary disabled:opacity-50"
          >
            {{ t('projects.startScan') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showGroupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showGroupDialog = false; selectedGroupId = ''; editingProjectId = null">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('projects.groupTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
          {{ t('projects.groupDesc') }}
        </p>
        <div class="space-y-2 max-h-60 overflow-y-auto">
          <button
            @click="selectedGroupId = ''"
            :class="[
              'w-full flex items-center gap-2 p-2.5 rounded-[6px] border transition-colors text-left',
              !selectedGroupId ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40 hover:border-primary-300 dark:hover:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer'
            ]"
          >
            <svg class="w-4 h-4 text-gray-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" /></svg>
            <span class="text-sm text-gray-500 dark:text-content-muted">{{ t('projects.ungrouped') }}</span>
          </button>
          <div
            v-for="group in availableGroups"
            :key="group.group_id"
            @click="selectedGroupId = group.group_id"
            :class="[
              'w-full flex items-center gap-2 p-2.5 rounded-[6px] border transition-colors text-left cursor-pointer',
              selectedGroupId === group.group_id ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40 hover:border-primary-300 dark:hover:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer'
            ]"
          >
            <span v-if="group.icon" class="text-base shrink-0">{{ group.icon }}</span>
            <span v-else class="w-4 h-4 rounded-full shrink-0" :style="{ backgroundColor: group.color }"></span>
            <span class="text-sm font-medium text-gray-900 dark:text-content-primary flex-1 truncate">{{ group.name }}</span>
            <button
              @click.stop="deleteGroup(group.group_id)"
              class="text-gray-400 hover:text-red-500 dark:hover:text-red-400 p-0.5 shrink-0"
              :title="t('projects.deleteGroup')"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
            </button>
          </div>
        </div>
        <button
          @click="openCreateGroupDialog"
          class="w-full mt-2 py-2 text-sm text-primary-600 dark:text-brand-primary hover:bg-primary-50 dark:hover:bg-surface-hover rounded-btn border border-dashed border-primary-300 dark:border-surface-border flex items-center justify-center gap-1.5 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 4v16m8-8H4" /></svg>
          {{ t('projects.createGroup') }}
        </button>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showGroupDialog = false; selectedGroupId = ''; editingProjectId = null"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveGroup"
            class="btn-primary"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showBatchGroupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBatchGroupDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('projects.batchGroupTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
          {{ t('projects.batchGroupDesc', { count: selectedCount }) }}
        </p>
        <div class="space-y-2 max-h-60 overflow-y-auto">
          <button
            @click="batchGroupId = ''"
            :class="[
              'w-full flex items-center gap-2 p-2.5 rounded-[6px] border transition-colors text-left',
              !batchGroupId ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40 hover:border-primary-300 dark:hover:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer'
            ]"
          >
            <svg class="w-4 h-4 text-gray-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" /></svg>
            <span class="text-sm text-gray-500 dark:text-content-muted">{{ t('projects.ungrouped') }}</span>
          </button>
          <button
            v-for="group in availableGroups"
            :key="group.group_id"
            @click="batchGroupId = group.group_id"
            :class="[
              'w-full flex items-center gap-2 p-2.5 rounded-[6px] border transition-colors text-left',
              batchGroupId === group.group_id ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover' : 'border-gray-200/60 dark:border-surface-border/40 hover:border-primary-300 dark:hover:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer'
            ]"
          >
            <span v-if="group.icon" class="text-base shrink-0">{{ group.icon }}</span>
            <span v-else class="w-4 h-4 rounded-full shrink-0" :style="{ backgroundColor: group.color }"></span>
            <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ group.name }}</span>
          </button>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showBatchGroupDialog = false"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveBatchGroup"
            class="btn-primary"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showRelocateDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRelocateDialog = false">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('projects.relocateTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-4">
          {{ t('projects.relocateDesc') }}
        </p>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('projects.newPath') }}</label>
          <div class="flex gap-2">
            <input
              v-model="relocateNewPath"
              type="text"
              readonly
              :placeholder="t('projects.scanPlaceholder')"
              class="flex-1 input-field bg-gray-50 dark:bg-surface-hover"
            />
            <button
              @click="selectRelocatePath"
              class="btn-secondary text-sm whitespace-nowrap"
            >
              {{ t('projects.browse') }}
            </button>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showRelocateDialog = false"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="confirmRelocate"
            :disabled="!relocateNewPath"
            class="btn-primary disabled:opacity-50"
          >
            {{ t('projects.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <ConfirmDialog
    v-model="showConfirmDialog"
    :title="confirmAction?.title || ''"
    :description="confirmAction?.message || ''"
    :confirm-text="t('common.confirmDelete')"
    @confirm="onConfirmDialogConfirm"
  >
    <div v-if="batchDeleteProgress" class="mt-3">
      <div class="flex justify-between text-xs text-gray-500 dark:text-content-muted mb-1">
        <span>{{ batchDeleteProgress.message }}</span>
        <span>{{ batchDeleteProgress.current }} / {{ batchDeleteProgress.total }}</span>
      </div>
      <div class="w-full bg-gray-200 dark:bg-surface-hover rounded-full h-1.5">
        <div class="bg-primary-600 h-1.5 rounded-full transition-all duration-300" :style="{ width: `${(batchDeleteProgress.current / batchDeleteProgress.total) * 100}%` }"></div>
      </div>
    </div>
    <label v-else class="flex items-center gap-2 mt-2 cursor-pointer select-none">
      <input type="checkbox" v-model="deleteWithFiles" class="w-4 h-4 rounded-[3px] border border-gray-300 dark:border-surface-border text-red-500 focus:ring-2 focus:ring-red-500/20 bg-white dark:bg-surface-input cursor-pointer" />
      <span class="text-sm text-red-600 dark:text-red-400">{{ t('projects.deleteWithFiles') }}</span>
    </label>
  </ConfirmDialog>

  <ConfirmDialog
    v-model="showSingleDeleteConfirm"
    :title="t('common.confirmDelete')"
    :description="t('projects.deleteConfirm', { count: 1, name: singleDeleteTarget?.name || '' })"
    :confirm-text="t('common.confirmDelete')"
    confirm-color="red"
    @confirm="onSingleDeleteConfirm"
  >
    <label class="flex items-center gap-2 mt-2 cursor-pointer select-none">
      <input type="checkbox" v-model="deleteWithFiles" class="w-4 h-4 rounded-[3px] border border-gray-300 dark:border-surface-border text-red-500 focus:ring-2 focus:ring-red-500/20 bg-white dark:bg-surface-input cursor-pointer" />
      <span class="text-sm text-red-600 dark:text-red-400">{{ t('projects.deleteWithFiles') }}</span>
    </label>
  </ConfirmDialog>

  <ConfirmDialog
    v-model="showGroupDeleteConfirm"
    :title="t('common.confirmDelete')"
    :description="t('projects.deleteGroupConfirm', { name: groupDeleteTarget?.name || '' })"
    :confirm-text="t('common.confirmDelete')"
    confirm-color="red"
    @confirm="onGroupDeleteConfirm"
  />

  <Teleport to="body">
    <div v-if="showMovedDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showMovedDialog = false">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('projects.detectProjectMigration') }}</h3>
        <p class="text-sm text-gray-600 dark:text-content-muted mb-4">
          {{ t('projects.migrationDesc') }}
        </p>
        <div class="space-y-3 max-h-60 overflow-y-auto">
          <div v-for="candidate in movedCandidates" :key="candidate.project_id" class="bg-gray-50 dark:bg-surface-hover rounded-[6px] p-4">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-gray-900 dark:text-content-primary">{{ candidate.old_name }}</h4>
                <p class="text-xs text-red-500 dark:text-red-400 mt-1">{{ t('projects.oldPath') }}: {{ candidate.old_path }}</p>
                <p class="text-xs text-green-500 dark:text-green-400">{{ t('projects.newPath') }}: {{ candidate.new_path }}</p>
              </div>
              <div class="flex gap-2">
                <button
                  @click="confirmMovedProject(candidate)"
                  class="px-3 py-1 btn-primary text-sm"
                >
                  {{ t('projects.update') }}
                </button>
                <button
                  @click="dismissMovedProject(candidate)"
                  class="btn-secondary text-sm"
                >
                  {{ t('projects.ignore') }}
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-end mt-4">
          <button
            @click="showMovedDialog = false"
            class="btn-secondary"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showAddProjectDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="closeAddProjectDialog">
      <div class="dialog-container w-full max-w-lg" @click.stop>
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('projects.addProjectDialog.title') }}</h3>
          <button @click="closeAddProjectDialog" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" /></svg>
          </button>
        </div>

        <div class="flex border-b border-gray-200/60 dark:border-surface-border/40 mb-4">
          <button
            @click="addProjectMode = 'local'"
            :class="['px-4 py-2 text-sm font-medium border-b-2 transition-colors', addProjectMode === 'local' ? 'border-primary-600 text-primary-600 dark:border-brand-primary dark:text-brand-primary' : 'border-transparent text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-content-secondary']"
          >
            {{ t('projects.addProjectDialog.local') }}
          </button>
          <button
            @click="addProjectMode = 'git'"
            :class="['px-4 py-2 text-sm font-medium border-b-2 transition-colors', addProjectMode === 'git' ? 'border-primary-600 text-primary-600 dark:border-brand-primary dark:text-brand-primary' : 'border-transparent text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-content-secondary']"
          >
            {{ t('projects.addProjectDialog.git') }}
          </button>
          <button
            @click="addProjectMode = 'template'"
            :class="['px-4 py-2 text-sm font-medium border-b-2 transition-colors', addProjectMode === 'template' ? 'border-primary-600 text-primary-600 dark:border-brand-primary dark:text-brand-primary' : 'border-transparent text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-content-secondary']"
          >
            {{ t('projects.addProjectDialog.template') }}
          </button>
        </div>

        <div v-if="addProjectMode === 'local'" class="space-y-3">
          <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('projects.addLocalDesc') }}</p>
          <div class="flex justify-end">
            <button
              @click="closeAddProjectDialog(); addProject()"
              class="btn-primary text-sm"
            >
              {{ t('projects.addProjectDialog.selectDir') }}
            </button>
          </div>
        </div>

        <div v-if="addProjectMode === 'git'" class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.gitImport.urlLabel') }}</label>
            <input
              v-model="gitUrl"
              type="text"
              :placeholder="t('projects.gitImport.urlPlaceholder')"
              class="w-full input-field"
              @keyup.enter="importProjectFromGit"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.gitImport.targetDirLabel') }}</label>
            <div class="flex gap-2">
              <input
                v-model="gitTargetDir"
                type="text"
                :placeholder="t('projects.gitImport.targetDirPlaceholder')"
                class="flex-1 input-field"
              />
              <button
                @click="browseGitTargetDir"
                class="px-3 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-btn hover:bg-gray-50 dark:hover:bg-surface-layer text-sm"
              >
                {{ t('projects.browse') }}
              </button>
            </div>
            <p class="mt-1 text-xs text-gray-500 dark:text-content-muted">{{ t('projects.gitImport.targetDirHint') }}</p>
          </div>
          <div class="flex justify-end space-x-3">
            <button @click="closeAddProjectDialog" class="btn-secondary text-sm">{{ t('common.cancel') }}</button>
            <button
              @click="importProjectFromGit"
              :disabled="isCloningFromGit || !gitUrl.trim()"
              class="btn-primary disabled:opacity-50 text-sm"
            >
              {{ isCloningFromGit ? t('projects.gitImport.cloning') : t('projects.gitImport.clone') }}
            </button>
          </div>
        </div>

        <div v-if="addProjectMode === 'template'" class="space-y-3">
          <div v-if="templateList.length === 0 && hubTemplateList.length === 0" class="text-center py-6">
            <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('projects.addProjectDialog.noTemplates') }}</p>
            <button
              @click="closeAddProjectDialog(); $router.push('/templates')"
              class="mt-3 text-sm text-primary-600 hover:text-primary-800 dark:text-brand-primary"
            >
              {{ t('projects.addProjectDialog.manageTemplates') }} &rarr;
            </button>
          </div>
          <div v-else class="space-y-3">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.addProjectDialog.selectTemplate') }}</label>
              <div class="grid grid-cols-2 gap-2 max-h-40 overflow-y-auto">
                <button
                  v-for="tmpl in hubTemplateList"
                  :key="tmpl.template_id"
                  @click="selectedTemplateId = 'hub:' + tmpl.template_id"
                  :class="['p-2.5 rounded-[6px] border text-left transition-all', selectedTemplateId === 'hub:' + tmpl.template_id ? 'border-primary-600 bg-primary-50 dark:bg-primary-900/20 dark:border-brand-primary' : 'border-gray-200/60 dark:border-surface-border/40 hover:border-gray-300 dark:hover:border-surface-hover']"
                >
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ tmpl.name }}</div>
                  <div class="text-xs text-gray-500 dark:text-content-muted truncate">{{ tmpl.description || t('projects.addProjectDialog.noDesc') }}</div>
                </button>
                <button
                  v-for="tmpl in templateList"
                  :key="tmpl.template_id"
                  @click="selectedTemplateId = tmpl.template_id"
                  :class="['p-2.5 rounded-[6px] border text-left transition-all', selectedTemplateId === tmpl.template_id ? 'border-primary-600 bg-primary-50 dark:bg-primary-900/20 dark:border-brand-primary' : 'border-gray-200/60 dark:border-surface-border/40 hover:border-gray-300 dark:hover:border-surface-hover']"
                >
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ tmpl.name }}</div>
                  <div class="text-xs text-gray-500 dark:text-content-muted">{{ t('projects.addProjectDialog.localTemplate') }}</div>
                </button>
              </div>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.addProjectDialog.projectName') }}</label>
              <input
                v-model="templateProjectName"
                type="text"
                :placeholder="t('projects.addProjectDialog.projectNamePlaceholder')"
                class="w-full input-field"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.addProjectDialog.targetDir') }}</label>
              <div class="flex gap-2">
                <input
                  v-model="templateTargetDir"
                  type="text"
                  :placeholder="t('projects.gitImport.targetDirPlaceholder')"
                  class="flex-1 input-field"
                />
                <button
                  @click="browseTemplateTargetDir"
                  class="px-3 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-btn hover:bg-gray-50 dark:hover:bg-surface-layer text-sm"
                >
                  {{ t('projects.browse') }}
                </button>
              </div>
            </div>
            <div class="flex justify-between items-center pt-2">
              <button
                @click="closeAddProjectDialog(); $router.push('/templates')"
                class="text-xs text-primary-600 hover:text-primary-800 dark:text-brand-primary"
              >
                {{ t('projects.addProjectDialog.manageTemplates') }}
              </button>
              <div class="flex space-x-3">
                <button @click="closeAddProjectDialog" class="btn-secondary text-sm">{{ t('common.cancel') }}</button>
                <button
                  @click="createProjectFromTemplate"
                  :disabled="isCreatingFromTemplate || !selectedTemplateId || !templateProjectName.trim()"
                  class="btn-primary disabled:opacity-50 text-sm"
                >
                  {{ isCreatingFromTemplate ? t('common.loading') : t('projects.addProjectDialog.create') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showGitDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showGitDialog = false; gitUrl = ''; gitTargetDir = ''">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('projects.gitImport.title') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">{{ t('projects.gitImport.desc') }}</p>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.gitImport.urlLabel') }}</label>
            <input
              v-model="gitUrl"
              type="text"
              :placeholder="t('projects.gitImport.urlPlaceholder')"
              class="w-full input-field"
              @keyup.enter="importProjectFromGit"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.gitImport.targetDirLabel') }}</label>
            <div class="flex gap-2">
              <input
                v-model="gitTargetDir"
                type="text"
                :placeholder="t('projects.gitImport.targetDirPlaceholder')"
                class="flex-1 input-field"
              />
              <button
                @click="browseGitTargetDir"
                class="px-3 py-2 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-hover text-gray-700 dark:text-content-secondary rounded-btn hover:bg-gray-50 dark:hover:bg-surface-layer text-sm"
              >
                {{ t('projects.browse') }}
              </button>
            </div>
            <p class="mt-1 text-xs text-gray-500 dark:text-content-muted">{{ t('projects.gitImport.targetDirHint') }}</p>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showGitDialog = false; gitUrl = ''; gitTargetDir = ''"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="importProjectFromGit"
            :disabled="isCloningFromGit || !gitUrl.trim()"
            class="btn-primary disabled:opacity-50"
          >
            {{ isCloningFromGit ? t('projects.gitImport.cloning') : t('projects.gitImport.clone') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showEngineSelectDialog && engineSelectProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="closeEngineSelectDialog">
      <div class="dialog-container w-full max-w-md max-h-[80vh] flex flex-col" @click.stop>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-1">{{ engineSelectMode === 'select' ? t('projects.selectDefaultEngine') : t('projects.openWithEngine') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
          {{ engineSelectMode === 'select' ? t('projects.selectDefaultEngineDesc') : t('projects.openWithEngineDesc') }}
          <span class="font-mono text-xs bg-gray-100 dark:bg-surface-hover px-1.5 py-0.5 rounded-[4px] ml-1">Godot {{ engineSelectProject.godot_version }}</span>
        </p>

        <div v-if="isLoadingEngines" class="flex-1 flex items-center justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
        </div>

        <div v-else-if="matchedEngines.length === 0" class="flex-1 py-6 text-center">
          <svg class="mx-auto h-10 w-10 text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('projects.noMatchingEngines') }}</p>
          <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('projects.noMatchingEnginesDesc') }}</p>
          <div class="flex items-center justify-center gap-3 mt-4">
            <button
              @click="closeEngineSelectDialog(); $router.push('/engines')"
              class="px-4 py-2 btn-primary text-sm transition-colors"
            >
              {{ t('projects.goToEngines') }}
            </button>
            <button
              @click="closeEngineSelectDialog(); $router.push('/engines?action=discover')"
              class="px-4 py-2 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary text-sm rounded-btn hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors"
            >
              {{ t('projects.discoverEngines') }}
            </button>
          </div>
        </div>

        <div v-else class="flex-1 overflow-y-auto space-y-2 min-h-0">
          <button
            v-for="me in matchedEngines"
            :key="me.engine.engine_id"
            @click="launchWithEngine(me.engine.engine_id)"
            :disabled="isLaunching"
            :class="[
              'w-full text-left p-3 rounded-[6px] border transition-colors disabled:opacity-40 disabled:cursor-not-allowed',
              me.engine.engine_id === engineSelectProject?.last_used_engine_id
                ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover'
                : 'border-gray-200/60 dark:border-surface-border/40 hover:border-primary-300 dark:hover:border-surface-border hover:bg-primary-50 dark:hover:bg-surface-hover'
            ]"
          >
            <div class="flex items-center justify-between">
              <div class="min-w-0 flex-1">
                <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1.5">
                  {{ me.engine.name }}
                  <span v-if="me.engine.engine_id === engineSelectProject?.last_used_engine_id" class="text-xs text-primary-600 dark:text-brand-primary font-normal">{{ t('projects.lastUsedEngine') }}</span>
                </div>
                <div class="text-xs text-gray-500 dark:text-content-muted mt-0.5 font-mono flex items-center gap-1.5">v{{ me.engine.version }}<span v-if="me.engine.is_mono" class="text-[10px] px-1 py-0.5 rounded-[4px] bg-purple-100 dark:bg-surface-hover text-purple-700 dark:text-content-secondary font-sans font-medium">{{ t('projects.monoLabel') }}</span></div>
              </div>
              <span
                :class="['text-xs px-2 py-0.5 rounded-full font-medium ml-2 flex-shrink-0', getMatchLevelClass(me.match_level)]"
                :title="getMatchLevelDesc(me.match_level)"
              >
                {{ getMatchLevelLabel(me.match_level) }}
              </span>
            </div>
            <div v-if="me.match_level !== 'exact'" class="mt-1.5 text-xs text-yellow-600 dark:text-yellow-400 flex items-center gap-1">
              <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" /></svg>
              {{ getMatchLevelDesc(me.match_level) }}
            </div>
          </button>
        </div>

        <div class="flex justify-end mt-4 pt-3 border-t border-gray-200/60 dark:border-surface-border/40">
          <button
            @click="closeEngineSelectDialog"
            class="btn-secondary"
          >
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showSaveAsTemplateDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="!isSavingAsTemplate && (showSaveAsTemplateDialog = false)">
      <div class="dialog-container max-w-md w-full mx-4" @click.stop>
        <div class="p-4">
          <h2 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('templates.saveFromProject') }}</h2>
          <p class="text-sm text-gray-500 dark:text-content-muted mb-3">{{ t('templates.saveFromProjectDesc') }}</p>
          <div class="space-y-3">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.templateName') }}</label>
              <input
                v-model="saveAsTemplateName"
                type="text"
                :disabled="isSavingAsTemplate"
                class="w-full input-field text-sm disabled:opacity-50"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.templateCategory') }}</label>
              <select
                v-model="saveAsTemplateCategory"
                :disabled="isSavingAsTemplate"
                class="w-full input-field text-sm disabled:opacity-50"
              >
                <option value="Starter2D">{{ t('templates.category.Starter2D') }}</option>
                <option value="Starter3D">{{ t('templates.category.Starter3D') }}</option>
                <option value="RPG">{{ t('templates.category.RPG') }}</option>
                <option value="Platformer">{{ t('templates.category.Platformer') }}</option>
                <option value="Multiplayer">{{ t('templates.category.Multiplayer') }}</option>
                <option value="Mobile">{{ t('templates.category.Mobile') }}</option>
                <option value="Blank">{{ t('templates.category.Blank') }}</option>
                <option value="Custom">{{ t('templates.category.Custom') }}</option>
              </select>
            </div>
          </div>
          <div class="flex gap-3 mt-6">
            <button
              @click="showSaveAsTemplateDialog = false"
              :disabled="isSavingAsTemplate"
              class="flex-1 py-2.5 text-sm font-medium rounded-btn border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
            >
              {{ t('common.cancel') || 'Cancel' }}
            </button>
            <button
              @click="handleSaveAsTemplate"
              :disabled="isSavingAsTemplate || !saveAsTemplateName.trim()"
              class="flex-1 py-2.5 text-sm font-medium btn-primary transition-colors disabled:opacity-50"
            >
              {{ isSavingAsTemplate ? '...' : t('common.save') || 'Save' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Create Group Dialog -->
  <Teleport to="body">
    <div v-if="showCreateGroupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="!isCreatingGroup && (showCreateGroupDialog = false)">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="dialog-title">{{ t('projects.createGroup') }}</h3>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.groupName') }}</label>
            <input
              v-model="newGroupName"
              type="text"
              :placeholder="t('projects.groupNamePlaceholder')"
              class="w-full input-field"
              @keyup.enter="createGroup"
            />
          </div>
          <div class="flex gap-3">
            <div class="flex-1">
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.groupIcon') }}</label>
              <input
                v-model="newGroupIcon"
                type="text"
                placeholder="🎮"
                class="w-full input-field"
              />
            </div>
            <div class="flex-1">
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.groupColor') }}</label>
              <div class="flex items-center gap-2">
                <input
                  v-model="newGroupColor"
                  type="color"
                  class="w-8 h-8 rounded-[4px] border border-gray-300 dark:border-surface-border cursor-pointer"
                />
                <input
                  v-model="newGroupColor"
                  type="text"
                  class="flex-1 input-field font-mono"
                />
              </div>
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.groupDescription') }}</label>
            <input
              v-model="newGroupDescription"
              type="text"
              :placeholder="t('projects.groupDescriptionPlaceholder')"
              class="w-full input-field"
            />
          </div>
          <div class="p-3 rounded-[6px] border border-gray-200/60 dark:border-surface-border/40 bg-gray-50 dark:bg-surface-hover">
            <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('projects.preview') }}</p>
            <div class="flex items-center gap-2">
              <span v-if="newGroupIcon" class="text-base">{{ newGroupIcon }}</span>
              <span class="w-3 h-3 rounded-full" :style="{ backgroundColor: newGroupColor }"></span>
              <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ newGroupName || t('projects.groupName') }}</span>
            </div>
          </div>
        </div>
        <div class="flex gap-3 mt-6">
          <button
            @click="showCreateGroupDialog = false"
            :disabled="isCreatingGroup"
            class="flex-1 py-2.5 text-sm font-medium rounded-btn border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="createGroup"
            :disabled="isCreatingGroup || !newGroupName.trim()"
            class="flex-1 py-2.5 text-sm font-medium btn-primary transition-colors disabled:opacity-50"
          >
            {{ isCreatingGroup ? '...' : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <ContextMenu
    :visible="projectContextMenu.visible.value"
    :x="projectContextMenu.x.value"
    :y="projectContextMenu.y.value"
    :items="projectContextMenu.items.value"
    @close="projectContextMenu.close()"
  />

</template>
