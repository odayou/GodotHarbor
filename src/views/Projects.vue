<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import type { Project, Engine, ProjectEngineBinding, MovedProjectCandidate, ProjectBinding, Plugin } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useBatchSelection } from '@/composables/useBatchSelection'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useAutoSetup } from '@/composables/useAutoSetup'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const router = useRouter()
const toast = useToast()
const { t } = useI18n()
const { runAutoSetup } = useAutoSetup()
const projects = ref<Project[]>([])
const engines = ref<Engine[]>([])
const projectBindingMap = ref<Map<string, ProjectBinding[]>>(new Map())
const projectEngineMap = ref<Map<string, string>>(new Map())
const isLoading = ref(false)
const showScanDialog = ref(false)
const scanDirInput = ref('')
const showProjectDetail = ref(false)
const selectedProject = ref<Project | null>(null)
const showGroupDialog = ref(false)
const groupInput = ref('')
const editingProjectId = ref<string | null>(null)
const showEngineDialog = ref(false)
const selectedEngineId = ref<string>('')
const customArgs = ref('')
const isLaunching = ref(false)
const projectEngineBinding = ref<ProjectEngineBinding | null>(null)
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
const filterGroup = ref<string>('all')
const filterStatus = ref<string>('all')
const availableGroups = ref<string[]>([])
let unlisten: UnlistenFn | null = null
let unlistenFs: UnlistenFn | null = null

const sortBy = ref<string>('name')
const sortOrder = ref<string>('asc')

const showBatchGroupDialog = ref(false)
const batchGroupInput = ref('')

const batchRemoveProjects = async () => {
  const ids = Array.from(selectedProjectIds.value)
  if (ids.length === 0) return
  confirm(t('common.confirmDelete'), t('projects.deleteConfirm', { count: ids.length }), async () => {
    try {
      const result = await api.batchRemoveProjects(ids)
      if (result.failed_count > 0) {
        toast.warning(t('common.batchDeleteComplete', { success: result.success_count, failed: result.failed_count }))
      } else {
        toast.success(t('common.batchDeleteSuccess', { count: result.success_count }))
      }
      clearSelection()
      await loadProjects()
    } catch (error) {
      toast.error(t('common.batchDeleteFailed', { error }))
    }
  })
}

const batchSetGroup = async () => {
  const ids = Array.from(selectedProjectIds.value)
  if (ids.length === 0) return
  batchGroupInput.value = ''
  showBatchGroupDialog.value = true
}

const saveBatchGroup = async () => {
  const ids = Array.from(selectedProjectIds.value)
  const results = await Promise.allSettled(
    ids.map(id => api.updateProjectGroup(id, batchGroupInput.value))
  )
  const successCount = results.filter(r => r.status === 'fulfilled').length
  const failCount = results.filter(r => r.status === 'rejected').length
  if (failCount === 0) {
    toast.success(t('projects.batchGroupSuccess', { count: successCount }))
  } else {
    toast.warning(t('projects.batchGroupPartial', { success: successCount, failed: failCount }))
  }
  showBatchGroupDialog.value = false
  clearSelection()
  await loadProjects()
}

onMounted(async () => {
  loadProjects()
  loadGroups()
  loadEngines()
  unlisten = await listen('scan-complete', () => {
    loadProjects()
  })
  unlistenFs = await listen('project-fs-changed', async () => {
    try {
      const synced = await api.syncProjects()
      projects.value = synced
    } catch (error) {
      console.error('增量同步失败:', error)
    }
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (unlistenFs) {
    unlistenFs()
  }
})

const getIconUrl = (iconPath: string) => {
  if (!iconPath) return ''
  try {
    return convertFileSrc(iconPath)
  } catch {
    return ''
  }
}

const matchesSearch = (project: Project) =>
  searchQuery.value === '' ||
  project.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
  project.path.toLowerCase().includes(searchQuery.value.toLowerCase())

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

const showProjectDetails = async (project: Project) => {
  selectedProject.value = project
  showProjectDetail.value = true
  try {
    projectBindings.value = await api.getProjectBindings(project.project_id)
    projectEngineBinding.value = await api.getProjectEngineBinding(project.project_id)
    if (projectEngineBinding.value) {
      selectedEngineId.value = projectEngineBinding.value.engine_id
      customArgs.value = projectEngineBinding.value.custom_args
    } else {
      selectedEngineId.value = ''
      customArgs.value = ''
    }
  } catch (error) {
    console.error('Failed to load project details:', error)
  }
}

const loadProjects = async () => {
  isLoading.value = true
  try {
    const result = await api.getProjects()
    projects.value = result
    await loadGroups()
    await checkMovedProjects()
    await loadAllProjectBindings()
    try {
      allPlugins.value = await api.getPlugins()
    } catch {
      allPlugins.value = []
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const loadAllProjectBindings = async () => {
  const map = new Map<string, ProjectBinding[]>()
  const engineMap = new Map<string, string>()
  const bindingResults = await Promise.allSettled(
    projects.value.map(p => api.getProjectBindings(p.project_id))
  )
  const engineResults = await Promise.allSettled(
    projects.value.map(p => api.getProjectEngineBinding(p.project_id))
  )
  bindingResults.forEach((result, i) => {
    map.set(projects.value[i].project_id, result.status === 'fulfilled' ? result.value : [])
  })
  engineResults.forEach((result, i) => {
    if (result.status === 'fulfilled' && result.value) {
      const engine = engines.value.find(e => e.engine_id === result.value!.engine_id)
      engineMap.set(projects.value[i].project_id, engine?.name || result.value!.engine_id)
    }
  })
  projectBindingMap.value = map
  projectEngineMap.value = engineMap
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
    const rootDirs = settings.scan_directories.length > 0 ? settings.scan_directories : []
    if (rootDirs.length === 0) {
      toast.warning(t('projects.noScanDirs'))
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

const addProject = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('projects.scanTitle')
    })
    if (selected && typeof selected === 'string') {
      const existing = projects.value.find(p => p.path === selected)
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

const showConfirmDialog = ref(false)
const confirmAction = ref<{ title: string; message: string; onConfirm: () => void } | null>(null)

const confirm = (title: string, message: string, onConfirm: () => void) => {
  confirmAction.value = { title, message, onConfirm }
  showConfirmDialog.value = true
}

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
    if (path && !projects.value.find(p => p.path === path)) {
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
  confirm(t('common.confirmDelete'), t('projects.deleteConfirm', { count: 1, name }), async () => {
    try {
      const bindings = projectBindingMap.value.get(projectId) || []
      for (const binding of bindings) {
        try {
          await api.unbindPlugin(projectId, binding.plugin_id)
        } catch { /* ignore unbind errors during removal */ }
      }
      try {
        await api.unbindProjectEngine(projectId)
      } catch { /* ignore engine unbind errors */ }
      await api.removeProject(projectId)
      toast.success(t('common.projectDeleted'))
      await loadProjects()
    } catch (error) {
      toast.error(t('common.deleteFailed', { error }))
    }
  })
}

const openInFileManager = async (path: string) => {
  try {
    await api.openInFileManager(path)
  } catch (error) {
    toast.error(t('projects.openInFileManagerFailed', { error }))
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

const openGroupDialog = (project: Project) => {
  editingProjectId.value = project.project_id
  groupInput.value = project.group || ''
  showGroupDialog.value = true
}

const saveGroup = async () => {
  if (!editingProjectId.value) return

  try {
    await api.updateProjectGroup(editingProjectId.value, groupInput.value)
    const project = projects.value.find(p => p.project_id === editingProjectId.value)
    if (project) {
      project.group = groupInput.value
    }
    toast.success(t('common.groupUpdated'))
    showGroupDialog.value = false
    editingProjectId.value = null
    groupInput.value = ''
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

const loadProjectEngineBinding = async (projectId: string) => {
  try {
    projectEngineBinding.value = await api.getProjectEngineBinding(projectId)
    if (projectEngineBinding.value) {
      selectedEngineId.value = projectEngineBinding.value.engine_id
      customArgs.value = projectEngineBinding.value.custom_args
    }
  } catch (error) {
    console.error('Failed to load engine binding:', error)
  }
}

const openEngineDialog = async (project: Project) => {
  selectedProject.value = project
  await loadProjectEngineBinding(project.project_id)
  showEngineDialog.value = true
}

const bindEngine = async () => {
  if (!selectedProject.value || !selectedEngineId.value) {
    toast.warning(t('common.selectEngine'))
    return
  }
  try {
    await api.bindProjectEngine(selectedProject.value.project_id, selectedEngineId.value, customArgs.value)
    toast.success(t('common.engineBindSuccess'))
    showEngineDialog.value = false
  } catch (error) {
    toast.error(t('common.engineBindFailed', { error }))
  }
}

const unbindEngine = async () => {
  if (!selectedProject.value) return
  try {
    await api.unbindProjectEngine(selectedProject.value.project_id)
    toast.success(t('common.engineUnbindSuccess'))
    projectEngineBinding.value = null
    selectedEngineId.value = ''
    customArgs.value = ''
  } catch (error) {
    toast.error(t('common.engineUnbindFailed', { error }))
  }
}

const launchProject = async (project: Project, engineId?: string) => {
  isLaunching.value = true
  try {
    if (!engineId) {
      const engineBinding = await api.getProjectEngineBinding(project.project_id)
      if (engineBinding) {
        engineId = engineBinding.engine_id
      } else {
        const defaultEngine = engines.value.find(e => e.is_default)
        if (!defaultEngine) {
          toast.warning(t('projects.noEngineHint'))
          isLaunching.value = false
          return
        }
        engineId = defaultEngine.engine_id
      }
    }
    try {
      const healthy = await api.checkEngineHealth(engineId)
      if (!healthy) {
        toast.error(t('projects.engineUnhealthy'))
        isLaunching.value = false
        return
      }
    } catch {
      // health check failed, proceed anyway
    }
    const result = await api.launchProjectWithEngine(project.project_id, engineId)
    if (result.success) {
      toast.success(t('common.projectLaunched', { pid: result.pid }))
    } else {
      toast.error(result.error || t('common.launchFailed'))
    }
  } catch (error) {
    toast.error(t('common.projectLaunchFailed', { error }))
  } finally {
    isLaunching.value = false
  }
}

const goToEngines = () => {
  router.push('/engines')
}

const showRelocateDialog = ref(false)
const relocateProjectId = ref('')
const relocateNewPath = ref('')

useDialogEscape(showScanDialog)
useDialogEscape(showProjectDetail)
useDialogEscape(showGroupDialog)
useDialogEscape(showEngineDialog)
useDialogEscape(showRelocateDialog)
useDialogEscape(showMovedDialog)
useDialogEscape(showBatchGroupDialog)

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
  router.push({ path: '/plugins', query: { tab: 'bindings', project: project.project_id } })
}

const bindPluginToProject = (project: Project) => {
  showProjectDetail.value = false
  router.push({ path: '/plugins', query: { tab: 'repository', bindProject: project.project_id } })
}

const unbindProjectBinding = async (binding: ProjectBinding) => {
  try {
    await api.unbindPlugin(binding.project_id, binding.plugin_id)
    try {
      await api.applyChanges(binding.project_id)
    } catch (applyErr) {
      toast.warning(t('linker.bindingApplyFailed', { errors: applyErr instanceof Error ? applyErr.message : String(applyErr) }))
    }
    toast.success(t('linker.pluginUnbound'))
    if (selectedProject.value) {
      projectBindings.value = await api.getProjectBindings(selectedProject.value.project_id)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const repairProjectBinding = async (binding: ProjectBinding) => {
  try {
    await api.repairBinding(binding.project_id, binding.plugin_id)
    try {
      await api.applyChanges(binding.project_id)
    } catch {
      // ignore apply errors
    }
    toast.success(t('linker.repairSuccess'))
    if (selectedProject.value) {
      projectBindings.value = await api.getProjectBindings(selectedProject.value.project_id)
    }
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}
</script>

<template>
  <div class="relative">
    <div v-if="isDragging" class="fixed inset-0 bg-primary-500/10 border-4 border-dashed border-primary-500 z-40 flex items-center justify-center pointer-events-none">
      <div class="bg-white dark:bg-gray-800 rounded-xl p-8 shadow-2xl">
        <svg class="mx-auto h-12 w-12 text-primary-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <p class="text-lg font-semibold text-primary-600 dark:text-primary-400">{{ t('projects.dragTitle') }}</p>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{{ t('projects.dragDesc') }}</p>
      </div>
    </div>
    <div
      class="space-y-6"
      @dragenter="onDragEnter"
      @dragleave="onDragLeave"
      @dragover="onDragOver"
      @drop="onDrop"
    >
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('projects.title') }}</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="showScanDialog = true"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('projects.scan') }}
        </button>
        <button
          @click="quickScan"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('projects.quickScan') }}
        </button>
        <button
          @click="addProject"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('projects.add') }}
        </button>
      </div>
    </div>

    <div class="card">
      <div class="flex flex-col lg:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('projects.search')"
            class="w-full px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          />
        </div>
        <div class="flex flex-wrap gap-2 items-center">
          <select
            v-model="filterGroup"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">{{ t('projects.allGroups') }}</option>
            <option value="ungrouped">{{ t('projects.ungrouped') }}</option>
            <option v-for="group in availableGroups" :key="group" :value="group">{{ group }}</option>
          </select>
          <select
            v-model="filterStatus"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
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
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="name">{{ t('projects.sortByName') }}</option>
            <option value="path">{{ t('projects.sortByPath') }}</option>
            <option value="godotVersion">{{ t('projects.sortByVersion') }}</option>
            <option value="status">{{ t('projects.sortByStatus') }}</option>
            <option value="updatedAt">{{ t('projects.sortByUpdated') }}</option>
          </select>
          <button
            @click="sortOrder = sortOrder === 'asc' ? 'desc' : 'asc'"
            class="px-2 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 text-sm"
            :title="sortOrder === 'asc' ? t('projects.ascending') : t('projects.descending')"
          >
            <svg v-if="sortOrder === 'asc'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
            </svg>
            <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div v-if="isBatchMode && selectedCount > 0" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-lg p-3 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <span class="text-sm font-medium text-primary-700 dark:text-primary-300">{{ t('projects.selectedCount', { count: selectedCount }) }}</span>
        <button
          @click="selectAllProjects"
          class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
        >
          {{ t('common.selectAll') }}
        </button>
        <button
          @click="clearSelection"
          class="text-xs text-gray-500 dark:text-gray-400 hover:underline"
        >
          {{ t('common.deselectAll') }}
        </button>
      </div>
      <div class="flex gap-2">
        <button
          @click="batchSetGroup"
          class="px-3 py-1.5 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors"
        >
          {{ t('projects.batchSetGroup') }} ({{ selectedCount }})
        </button>
        <button
          @click="batchRemoveProjects"
          class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-lg hover:bg-red-700 transition-colors"
        >
          {{ t('common.batchDelete') }} ({{ selectedCount }})
        </button>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="filteredProjects.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400 dark:text-content-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('projects.empty') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">
        {{ t('projects.emptyDesc') }}
      </p>
      <div class="mt-4 flex justify-center gap-3">
        <button
          @click="showScanDialog = true"
          :disabled="isLoading"
          class="inline-flex items-center gap-1.5 btn-primary disabled:opacity-50 text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          {{ t('projects.scan') }}
        </button>
        <button
          @click="addProject"
          :disabled="isLoading"
          class="inline-flex items-center gap-1.5 btn-secondary disabled:opacity-50 text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t('projects.add') }}
        </button>
      </div>
    </div>

    <div v-else class="space-y-6">
      <div v-for="(groupProjects, groupName) in (filterGroup === 'all' ? groupedProjects : { all: filteredProjects })" :key="groupName" class="space-y-3">
        <div v-if="filterGroup === 'all' && Object.keys(groupedProjects).length > 1" class="flex items-center gap-2">
          <h2 class="text-lg font-semibold text-gray-700 dark:text-content-primary">
            {{ groupName === t('projects.ungrouped') ? t('projects.ungrouped') : groupName }}
          </h2>
          <span class="text-sm text-gray-500 dark:text-content-secondary">({{ groupProjects.length }} {{ t('projects.projectCount') }})</span>
        </div>
        <div class="space-y-3">
          <div
            v-for="project in (filterGroup === 'all' ? groupProjects : filteredProjects)"
            :key="project.project_id"
            :class="[
              'bg-white dark:bg-surface-card rounded-xl shadow hover:shadow-md transition-shadow p-4 flex items-center gap-4',
              selectedProjectIds.has(project.project_id) ? 'ring-2 ring-primary-500' : ''
            ]"
          >
            <input
              type="checkbox"
              :checked="selectedProjectIds.has(project.project_id)"
              @click.stop="toggleProjectSelection(project, $event)"
              class="w-4 h-4 text-primary-600 rounded flex-shrink-0 cursor-pointer"
            />
            <div class="w-10 h-10 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700 flex items-center justify-center flex-shrink-0">
              <img
                v-if="project.icon_path"
                :src="getIconUrl(project.icon_path)"
                :alt="project.name"
                class="w-10 h-10 object-contain"
                @error="($event.target as HTMLImageElement).style.display = 'none'; ($event.target as HTMLImageElement).nextElementSibling?.classList.remove('hidden')"
              />
              <svg :class="project.icon_path ? 'hidden' : ''" class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <div 
              class="min-w-0 flex-1 cursor-pointer hover:text-primary-600 dark:hover:text-primary-400"
              @click="showProjectDetails(project)"
            >
              <div class="flex items-center gap-2">
                <h3 class="text-base font-semibold text-gray-900 dark:text-content-primary">
                  {{ project.name }}
                </h3>
                <span
                  v-if="project.group"
                  @click.stop="openGroupDialog(project)"
                  class="badge badge-neutral hover:bg-gray-200 dark:hover:bg-surface-layer cursor-pointer"
                >
                  {{ project.group }}
                </span>
                <span
                  :class="[
                    'badge',
                    project.status === 'Ready' ? 'badge-success' :
                    project.status === 'Warning' ? 'badge-warning' :
                    project.status === 'Conflict' ? 'badge-error' :
                    project.status === 'MissingSource' ? 'badge-neutral' :
                    'badge-error'
                  ]"
                >
                  {{ t(`projects.status.${project.status.toLowerCase()}`) }}
                </span>
              </div>
              <div class="flex items-center gap-3 mt-1">
                <span class="text-sm text-gray-500 dark:text-content-secondary" :title="project.path">
                  {{ project.path }}
                </span>
                <span class="text-sm text-gray-400">|</span>
                <span class="text-sm text-gray-500 dark:text-content-secondary">Godot {{ project.godot_version }}</span>
                <span
                  v-if="projectEngineMap.get(project.project_id)"
                  class="text-sm text-primary-600 dark:text-primary-400 flex items-center gap-1"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  {{ projectEngineMap.get(project.project_id) }}
                </span>
                <span
                  v-if="projectBindingMap.get(project.project_id)?.length"
                  class="text-sm text-gray-500 dark:text-content-secondary flex items-center gap-1"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                  {{ projectBindingMap.get(project.project_id)!.length }}
                  <span
                    v-if="projectBindingMap.get(project.project_id)?.some(b => b.is_healthy === false)"
                    class="w-2 h-2 rounded-full bg-red-500"
                    :title="t('projects.unhealthyBindings')"
                  ></span>
                </span>
              </div>
            </div>
            <div class="flex items-center gap-1">
              <button
                @click.stop="openInFileManager(project.path)"
                class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                :title="t('projects.openInFileManager')"
              >
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
              </button>
              <button
                @click.stop="syncProject(project)"
                class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                :title="t('projects.syncProject')"
              >
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
              <button
                @click.stop="openGroupDialog(project)"
                class="text-blue-600 hover:text-blue-800 p-2 rounded-lg hover:bg-blue-50 dark:hover:bg-blue-900/20"
                :title="t('projects.setGroup')"
              >
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
                </svg>
              </button>
              <button
                v-if="project.status === 'MissingSource'"
                @click.stop="openRelocateDialog(project)"
                class="px-3 py-1.5 rounded-lg text-sm font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors"
                :title="t('projects.relocate')"
              >
                {{ t('projects.relocate') }}
              </button>
              <template v-else-if="engines.length === 0">
                <button
                  @click.stop="goToEngines"
                  class="px-3 py-1.5 rounded-lg text-sm font-medium bg-yellow-500 text-white hover:bg-yellow-600 transition-colors"
                  :title="t('projects.noEngineHint')"
                >
                  {{ t('projects.registerEngine') }}
                </button>
              </template>
              <button
                v-else
                @click.stop="launchProject(project)"
                :disabled="isLaunching"
                class="px-3 py-1.5 rounded-lg text-sm font-medium bg-primary-600 text-white hover:bg-primary-700 disabled:opacity-50 transition-colors"
                :title="t('projects.launch')"
              >
                {{ t('projects.launch') }}
              </button>
              <button
                @click.stop="removeProject(project.project_id)"
                class="text-red-600 hover:text-red-800 p-2 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20"
              >
                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    </div>
  </div>

  <Teleport to="body">
  <div v-if="showProjectDetail && selectedProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showProjectDetail = false; selectedProject = null">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-lg shadow-xl max-h-[90vh] overflow-y-auto" @click.stop>
        <div class="flex items-center gap-4 mb-4">
          <div class="flex-shrink-0 w-12 h-12 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700 flex items-center justify-center">
            <img
              v-if="selectedProject.icon_path"
              :src="getIconUrl(selectedProject.icon_path)"
              :alt="selectedProject.name"
              class="w-12 h-12 object-contain"
              @error="($event.target as HTMLImageElement).style.display = 'none'"
            />
            <svg v-else class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {{ selectedProject.name }}
            </h3>
            <span class="text-sm text-gray-500 dark:text-gray-400">
              Godot {{ selectedProject.godot_version }}
            </span>
          </div>
        </div>
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('projects.projectPath') }}</h4>
          <div class="flex items-center gap-2">
            <p class="text-sm text-gray-600 dark:text-gray-400 break-all bg-gray-50 dark:bg-gray-700 rounded-lg p-3 flex-1">
              {{ selectedProject.path }}
            </p>
            <button
              @click="openInFileManager(selectedProject.path)"
              class="text-primary-600 hover:text-primary-800 dark:text-primary-400 p-1 flex-shrink-0"
              :title="t('projects.openInFileManager')"
            >
              <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
            </button>
          </div>
        </div>
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('projects.statusLabel') }}</h4>
          <span
            :class="[
              'px-3 py-1 rounded text-sm font-medium',
              selectedProject.status === 'Ready' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' :
              selectedProject.status === 'Warning' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400' :
              selectedProject.status === 'Conflict' ? 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400' :
              selectedProject.status === 'MissingSource' ? 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300' :
              'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
            ]"
          >
            {{ t(`projects.status.${selectedProject.status.toLowerCase()}`) }}
          </span>
          <span v-if="selectedProject.last_synced_at" class="text-xs text-gray-400 dark:text-gray-500 ml-3">
            {{ t('projects.lastSynced') }} {{ new Date(selectedProject.last_synced_at).toLocaleString() }}
          </span>
        </div>
        <div class="mb-4">
          <div class="flex items-center justify-between mb-2">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">{{ t('projects.pluginBindings') }}</h4>
            <div class="flex items-center gap-2">
              <button
                @click="bindPluginToProject(selectedProject!)"
                class="px-2.5 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 transition-colors flex items-center gap-1"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                </svg>
                {{ t('linker.bindPlugins') }}
              </button>
              <button
                @click="goToPluginBindings(selectedProject!)"
                class="px-2.5 py-1 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 text-xs rounded hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center gap-1"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
                {{ t('linker.goToPluginEcosystem') }}
              </button>
            </div>
          </div>
          <div v-if="projectBindings.length === 0" class="text-sm text-gray-500 dark:text-gray-400">
            {{ t('projects.noBindings') }}
          </div>
          <div v-else class="space-y-2 max-h-48 overflow-y-auto">
            <div
              v-for="binding in projectBindings"
              :key="binding.plugin_id + binding.mount_path"
              class="flex items-center justify-between p-2 rounded-lg"
              :class="binding.is_healthy === false ? 'bg-red-50 dark:bg-red-900/10 border border-red-200 dark:border-red-800' : 'bg-gray-50 dark:bg-gray-700'"
            >
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <span v-if="binding.is_healthy === false" class="flex-shrink-0">
                  <svg class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                </span>
                <span v-else class="flex-shrink-0">
                  <svg class="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                </span>
                <div class="min-w-0 flex-1">
                  <span class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">{{ getPluginName(binding.plugin_id) }}</span>
                  <span class="text-xs text-gray-400 dark:text-gray-500 ml-1.5">{{ getPluginVersion(binding.plugin_id) }}</span>
                  <span class="text-xs text-gray-500 dark:text-gray-400 ml-2 font-mono">{{ binding.mount_path }}</span>
                </div>
              </div>
              <div class="flex items-center gap-1 ml-2 flex-shrink-0">
                <button
                  v-if="binding.is_healthy === false"
                  @click="repairProjectBinding(binding)"
                  class="px-2 py-1 text-xs text-orange-600 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/20 rounded"
                >
                  {{ t('plugins.bindDialog.repair') }}
                </button>
                <button
                  @click="unbindProjectBinding(binding)"
                  class="px-2 py-1 text-xs text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
                >
                  {{ t('linker.unbind') }}
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('projects.engineBinding') }}</h4>
          <div class="flex items-center gap-2">
            <button
              @click="openEngineDialog(selectedProject)"
              class="px-3 py-1 rounded text-sm font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors"
            >
              {{ projectEngineBinding ? t('projects.changeEngine') : t('projects.bind') }}
            </button>
            <span v-if="projectEngineBinding" class="text-sm text-gray-600 dark:text-gray-400">
              {{ engines.find(e => e.engine_id === projectEngineBinding?.engine_id)?.name || t('projects.bound') }}
            </span>
          </div>
        </div>
        <div class="flex justify-between gap-2">
          <div class="flex gap-2">
            <button
              @click="openInFileManager(selectedProject.path)"
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 text-sm"
            >
              {{ t('projects.openInFileManager') }}
            </button>
            <button
              @click="syncProject(selectedProject)"
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 text-sm"
            >
              {{ t('projects.syncProject') }}
            </button>
            <button
              @click="showProjectDetail = false; openRelocateDialog(selectedProject)"
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 text-sm"
            >
              {{ t('projects.relocate') }}
            </button>
          </div>
          <button
            @click="showProjectDetail = false; selectedProject = null"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showScanDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showScanDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('projects.scanTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          {{ t('projects.scanDesc') }}
        </p>
        <div class="flex gap-2 mb-6">
          <input
            v-model="scanDirInput"
            type="text"
            :placeholder="t('projects.scanPlaceholder')"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          />
          <button
            @click="selectScanDir"
            class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
          >
            {{ t('projects.browse') }}
          </button>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            @click="showScanDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="startScan"
            :disabled="!scanDirInput"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ t('projects.startScan') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showGroupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showGroupDialog = false; groupInput = ''; editingProjectId = null">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('projects.groupTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          {{ t('projects.groupDesc') }}
        </p>
        <input
          v-model="groupInput"
          type="text"
          :placeholder="t('projects.groupPlaceholder')"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
        />
        <div v-if="availableGroups.length > 0" class="mt-3">
          <p class="text-xs text-gray-500 dark:text-gray-400 mb-1">{{ t('projects.existingGroups') }}</p>
          <div class="flex flex-wrap gap-1">
            <button
              v-for="group in availableGroups"
              :key="group"
              @click="groupInput = group"
              class="px-2 py-1 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
            >
              {{ group }}
            </button>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            v-if="groupInput"
            @click="groupInput = ''"
            class="px-4 py-2 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded-lg hover:bg-red-200 dark:hover:bg-red-900/50"
          >
            {{ t('projects.clearGroup') }}
          </button>
          <div class="flex-1"></div>
          <button
            @click="showGroupDialog = false; groupInput = ''; editingProjectId = null"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveGroup"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showBatchGroupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBatchGroupDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('projects.batchGroupTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          {{ t('projects.batchGroupDesc', { count: selectedCount }) }}
        </p>
        <input
          v-model="batchGroupInput"
          type="text"
          :placeholder="t('projects.groupPlaceholder')"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
        />
        <div v-if="availableGroups.length > 0" class="mt-3">
          <p class="text-xs text-gray-500 dark:text-gray-400 mb-1">{{ t('projects.existingGroups') }}</p>
          <div class="flex flex-wrap gap-1">
            <button
              v-for="group in availableGroups"
              :key="group"
              @click="batchGroupInput = group"
              class="px-2 py-1 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
            >
              {{ group }}
            </button>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showBatchGroupDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveBatchGroup"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showEngineDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showEngineDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          {{ selectedProject?.name }} - {{ t('projects.engineBind') }}
        </h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('projects.selectEngine') }}</label>
            <select
              v-model="selectedEngineId"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            >
              <option value="">{{ t('projects.selectEnginePlaceholder') }}</option>
              <option v-for="engine in engines" :key="engine.engine_id" :value="engine.engine_id">
                {{ engine.name }} (v{{ engine.version }}) {{ engine.is_default ? `- ${t('engines.default')}` : '' }}
              </option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('projects.launchArgs') }}</label>
            <input
              v-model="customArgs"
              type="text"
              :placeholder="t('projects.launchArgsPlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
        </div>
        <div class="flex justify-between mt-6">
          <button
            v-if="projectEngineBinding"
            @click="unbindEngine"
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
          >
            {{ t('projects.unbind') }}
          </button>
          <div class="flex gap-2 ml-auto">
            <button
              @click="showEngineDialog = false"
              class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              @click="bindEngine"
              :disabled="!selectedEngineId"
              class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
            >
              {{ t('projects.bind') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showConfirmDialog"
      :title="confirmAction?.title || ''"
      :description="confirmAction?.message || ''"
      :confirm-text="t('common.confirmDelete')"
      @confirm="onConfirmDialogConfirm"
    />

  </Teleport>

  <Teleport to="body">
    <div v-if="showRelocateDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRelocateDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('projects.relocateTitle') }}</h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
          {{ t('projects.relocateDesc') }}
        </p>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('projects.newPath') }}</label>
          <div class="flex gap-2">
            <input
              v-model="relocateNewPath"
              type="text"
              readonly
              :placeholder="t('projects.scanPlaceholder')"
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
            <button
              @click="selectRelocatePath"
              class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
            >
              {{ t('projects.browse') }}
            </button>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showRelocateDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="confirmRelocate"
            :disabled="!relocateNewPath"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ t('projects.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div v-if="showMovedDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showMovedDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-lg shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">{{ t('projects.detectProjectMigration') }}</h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
          {{ t('projects.migrationDesc') }}
        </p>
        <div class="space-y-3 max-h-60 overflow-y-auto">
          <div v-for="candidate in movedCandidates" :key="candidate.project_id" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-gray-900 dark:text-gray-100">{{ candidate.old_name }}</h4>
                <p class="text-xs text-red-500 dark:text-red-400 mt-1">{{ t('projects.oldPath') }}: {{ candidate.old_path }}</p>
                <p class="text-xs text-green-500 dark:text-green-400">{{ t('projects.newPath') }}: {{ candidate.new_path }}</p>
              </div>
              <div class="flex gap-2">
                <button
                  @click="confirmMovedProject(candidate)"
                  class="px-3 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 text-sm"
                >
                  {{ t('projects.update') }}
                </button>
                <button
                  @click="dismissMovedProject(candidate)"
                  class="px-3 py-1 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded hover:bg-gray-300 dark:hover:bg-gray-500 text-sm"
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
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
