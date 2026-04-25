<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { api } from '@/api'
import type { Project, Engine, ProjectEngineBinding, MovedProjectCandidate } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const projects = ref<Project[]>([])
const engines = ref<Engine[]>([])
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

const searchQuery = ref('')
const filterGroup = ref<string>('all')
const filterStatus = ref<string>('all')
const availableGroups = ref<string[]>([])
let unlisten: UnlistenFn | null = null

const selectedProjectIds = ref<Set<string>>(new Set())
const lastClickedIndex = ref<number>(-1)
const isBatchMode = ref(false)

const toggleProjectSelection = (project: Project, event: MouseEvent | Event) => {
  const mouseEvent = event as MouseEvent
  const projectId = project.project_id
  const currentList = filteredProjects.value
  const currentIndex = currentList.findIndex(p => p.project_id === projectId)

  if (mouseEvent.shiftKey && lastClickedIndex.value >= 0) {
    const start = Math.min(lastClickedIndex.value, currentIndex)
    const end = Math.max(lastClickedIndex.value, currentIndex)
    for (let i = start; i <= end; i++) {
      selectedProjectIds.value.add(currentList[i].project_id)
    }
  } else if (mouseEvent.ctrlKey || mouseEvent.metaKey) {
    if (selectedProjectIds.value.has(projectId)) {
      selectedProjectIds.value.delete(projectId)
    } else {
      selectedProjectIds.value.add(projectId)
    }
  } else {
    if (selectedProjectIds.value.has(projectId)) {
      selectedProjectIds.value.delete(projectId)
      if (selectedProjectIds.value.size === 0) {
        isBatchMode.value = false
      }
    } else {
      selectedProjectIds.value.add(projectId)
      isBatchMode.value = true
    }
  }

  lastClickedIndex.value = currentIndex
  selectedProjectIds.value = new Set(selectedProjectIds.value)
}

const selectAllProjects = () => {
  for (const p of filteredProjects.value) {
    selectedProjectIds.value.add(p.project_id)
  }
  selectedProjectIds.value = new Set(selectedProjectIds.value)
  isBatchMode.value = true
}

const clearSelection = () => {
  selectedProjectIds.value.clear()
  selectedProjectIds.value = new Set(selectedProjectIds.value)
  isBatchMode.value = false
  lastClickedIndex.value = -1
}

const selectedCount = computed(() => selectedProjectIds.value.size)

const batchRemoveProjects = async () => {
  const ids = Array.from(selectedProjectIds.value)
  if (ids.length === 0) return
  confirm('批量删除项目', `确定要删除选中的 ${ids.length} 个项目吗？此操作仅从列表中移除，不会删除项目文件。`, async () => {
    try {
      const result = await api.batchRemoveProjects(ids)
      if (result.failed_count > 0) {
        toast.warning(`批量删除完成: 成功 ${result.success_count} 个, 失败 ${result.failed_count} 个`)
      } else {
        toast.success(`已成功删除 ${result.success_count} 个项目`)
      }
      clearSelection()
      await loadProjects()
    } catch (error) {
      toast.error(`批量删除项目失败: ${error}`)
    }
  })
}

onMounted(async () => {
  loadProjects()
  loadGroups()
  loadEngines()
  unlisten = await listen('scan-complete', () => {
    loadProjects()
  })
  listen('project-fs-changed', async () => {
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
})

const getIconUrl = (iconPath: string) => {
  if (!iconPath) return ''
  try {
    return convertFileSrc(iconPath)
  } catch {
    return ''
  }
}

const groupedProjects = computed(() => {
  const groups: Record<string, Project[]> = {}

  const filtered = projects.value.filter(project => {
    const matchesSearch = searchQuery.value === '' ||
      project.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      project.path.toLowerCase().includes(searchQuery.value.toLowerCase())
    return matchesSearch
  })

  filtered.forEach(project => {
    const groupKey = project.group || '未分组'
    if (!groups[groupKey]) {
      groups[groupKey] = []
    }
    groups[groupKey].push(project)
  })

  return groups
})

const filteredProjects = computed(() => {
  return projects.value.filter(project => {
    const matchesSearch = searchQuery.value === '' ||
      project.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      project.path.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesGroup = filterGroup.value === 'all' ||
      (filterGroup.value === 'ungrouped' && !project.group) ||
      project.group === filterGroup.value

    const matchesStatus = filterStatus.value === 'all' ||
      project.status === filterStatus.value

    return matchesSearch && matchesGroup && matchesStatus
  })
})

const loadGroups = async () => {
  try {
    availableGroups.value = await api.getProjectGroups()
  } catch (error) {
    console.error('Failed to load groups:', error)
  }
}

const loadProjects = async () => {
  isLoading.value = true
  try {
    const result = await api.getProjects()
    projects.value = result
    await loadGroups()
    await checkMovedProjects()
  } catch (error) {
    toast.error(`加载项目失败: ${error}`)
  } finally {
    isLoading.value = false
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
    toast.success(`项目 ${candidate.old_name} 已迁移到新路径`)
    movedCandidates.value = movedCandidates.value.filter(c => c.project_id !== candidate.project_id)
    if (movedCandidates.value.length === 0) {
      showMovedDialog.value = false
    }
    await loadProjects()
  } catch (error) {
    toast.error(`迁移失败: ${error}`)
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
      title: '选择扫描目录'
    })
    if (selected && typeof selected === 'string') {
      scanDirInput.value = selected
    }
  } catch (error) {
    toast.error(`选择目录失败: ${error}`)
  }
}

const startScan = async () => {
  if (!scanDirInput.value) {
    toast.warning('请先选择扫描目录')
    return
  }
  showScanDialog.value = false
  isLoading.value = true
  try {
    const result = await api.scanProjects([scanDirInput.value])
    projects.value = result
    toast.success(`扫描完成，发现 ${result.length} 个项目`)
    await loadProjects()
  } catch (error) {
    toast.error(`扫描项目失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const quickScan = async () => {
  isLoading.value = true
  try {
    const settings = await api.getSettings()
    const rootDirs = settings.scan_directories?.length ? settings.scan_directories : []
    if (rootDirs.length === 0) {
      toast.info('未配置扫描目录，请先在设置中添加或手动选择目录')
      showScanDialog.value = true
      isLoading.value = false
      return
    }
    const result = await api.scanProjects(rootDirs)
    projects.value = result
    toast.success(`扫描完成，发现 ${result.length} 个项目`)
    await loadProjects()
  } catch (error) {
    toast.error(`扫描项目失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const addProject = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Godot 项目目录'
    })
    if (selected && typeof selected === 'string') {
      isLoading.value = true
      const result = await api.addProject(selected)
      toast.success(`成功添加项目: ${result.name}`)
      await loadProjects()
    }
  } catch (error) {
    toast.error(`添加项目失败: ${error}`)
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

  for (let i = 0; i < files.length; i++) {
    const file = files[i]
    const path = (file as any).path
    if (!path) continue

    try {
      isLoading.value = true
      const result = await api.addProject(path)
      toast.success(`成功添加项目: ${result.name}`)
    } catch (error: any) {
      if (!String(error).includes('已存在')) {
        console.log('Skipped non-project path:', path)
      }
    } finally {
      isLoading.value = false
    }
  }
  await loadProjects()
}

const removeProject = async (projectId: string) => {
  const project = projects.value.find(p => p.project_id === projectId)
  const name = project?.name || projectId
  confirm('删除项目', `确定要删除项目 "${name}" 吗？此操作仅从列表中移除，不会删除项目文件。`, async () => {
    try {
      await api.removeProject(projectId)
      toast.success('项目已删除')
      await loadProjects()
    } catch (error) {
      toast.error(`删除项目失败: ${error}`)
    }
  })
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
    toast.success('分组已更新')
    showGroupDialog.value = false
    editingProjectId.value = null
    groupInput.value = ''
    await loadGroups()
  } catch (error) {
    toast.error(`更新分组失败: ${error}`)
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
    toast.warning('请选择引擎')
    return
  }
  try {
    await api.bindProjectEngine(selectedProject.value.project_id, selectedEngineId.value, customArgs.value)
    toast.success('引擎绑定成功')
    showEngineDialog.value = false
  } catch (error) {
    toast.error(`绑定引擎失败: ${error}`)
  }
}

const unbindEngine = async () => {
  if (!selectedProject.value) return
  try {
    await api.unbindProjectEngine(selectedProject.value.project_id)
    toast.success('已解除引擎绑定')
    projectEngineBinding.value = null
    selectedEngineId.value = ''
    customArgs.value = ''
  } catch (error) {
    toast.error(`解除绑定失败: ${error}`)
  }
}

const launchProject = async (project: Project, engineId?: string) => {
  isLaunching.value = true
  try {
    const result = await api.launchProjectWithEngine(project.project_id, engineId)
    if (result.success) {
      toast.success(`项目已启动 (PID: ${result.pid})`)
    } else {
      toast.error(result.error || '启动失败')
    }
  } catch (error) {
    toast.error(`启动项目失败: ${error}`)
  } finally {
    isLaunching.value = false
  }
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
      title: '选择项目新路径'
    })
    if (selected && typeof selected === 'string') {
      relocateNewPath.value = selected
    }
  } catch (error) {
    toast.error(`选择目录失败: ${error}`)
  }
}

const confirmRelocate = async () => {
  if (!relocateNewPath.value) {
    toast.warning('请选择新路径')
    return
  }
  try {
    await api.relocateProject(relocateProjectId.value, relocateNewPath.value)
    toast.success('项目路径已更新')
    showRelocateDialog.value = false
    await loadProjects()
  } catch (error) {
    toast.error(`重新定位失败: ${error}`)
  }
}
</script>

<template>
  <div
    class="space-y-6"
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
    @dragover="onDragOver"
    @drop="onDrop"
  >
    <div v-if="isDragging" class="fixed inset-0 bg-primary-500/10 border-4 border-dashed border-primary-500 z-40 flex items-center justify-center pointer-events-none">
      <div class="bg-white dark:bg-gray-800 rounded-xl p-8 shadow-2xl">
        <svg class="mx-auto h-12 w-12 text-primary-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <p class="text-lg font-semibold text-primary-600 dark:text-primary-400">拖放 Godot 项目目录到此处</p>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">将自动识别包含 project.godot 的目录</p>
      </div>
    </div>
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">项目管理</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="showScanDialog = true"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
        >
          扫描项目
        </button>
        <button
          @click="quickScan"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          快速扫描
        </button>
        <button
          @click="addProject"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          添加项目
        </button>
      </div>
    </div>

    <div class="card">
      <div class="flex flex-col lg:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索项目名称或路径..."
            class="w-full px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          />
        </div>
        <div class="flex flex-wrap gap-2 items-center">
          <select
            v-model="filterGroup"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">全部分组</option>
            <option value="ungrouped">未分组</option>
            <option v-for="group in availableGroups" :key="group" :value="group">{{ group }}</option>
          </select>
          <select
            v-model="filterStatus"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">全部状态</option>
            <option value="Ready">就绪</option>
            <option value="Warning">警告</option>
            <option value="Error">错误</option>
            <option value="Conflict">冲突</option>
            <option value="MissingSource">源缺失</option>
          </select>
        </div>
      </div>
    </div>

    <div v-if="isBatchMode && selectedCount > 0" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-lg p-3 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <span class="text-sm font-medium text-primary-700 dark:text-primary-300">已选择 {{ selectedCount }} 个项目</span>
        <button
          @click="selectAllProjects"
          class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
        >
          全选
        </button>
        <button
          @click="clearSelection"
          class="text-xs text-gray-500 dark:text-gray-400 hover:underline"
        >
          取消选择
        </button>
      </div>
      <div class="flex gap-2">
        <button
          @click="batchRemoveProjects"
          class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-lg hover:bg-red-700 transition-colors"
        >
          批量删除 ({{ selectedCount }})
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
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">暂无项目</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">
        开始扫描或手动添加 Godot 项目
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
          扫描项目
        </button>
        <button
          @click="addProject"
          :disabled="isLoading"
          class="inline-flex items-center gap-1.5 btn-secondary disabled:opacity-50 text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          添加项目
        </button>
      </div>
    </div>

    <div v-else class="space-y-6">
      <div v-for="(groupProjects, groupName) in (filterGroup === 'all' ? groupedProjects : { all: filteredProjects })" :key="groupName" class="space-y-3">
        <div v-if="filterGroup === 'all' && Object.keys(groupedProjects).length > 1" class="flex items-center gap-2">
          <h2 class="text-lg font-semibold text-gray-700 dark:text-content-primary">
            {{ groupName === '未分组' ? '未分组' : groupName }}
          </h2>
          <span class="text-sm text-gray-500 dark:text-content-secondary">({{ groupProjects.length }} 个项目)</span>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <div
            v-for="project in (filterGroup === 'all' ? groupProjects : filteredProjects)"
            :key="project.project_id"
            :class="[
              'bg-white dark:bg-surface-card rounded-xl shadow hover:shadow-md transition-shadow p-5',
              selectedProjectIds.has(project.project_id) ? 'ring-2 ring-primary-500' : ''
            ]"
          >
            <div class="flex items-start justify-between min-w-0">
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <input
                  type="checkbox"
                  :checked="selectedProjectIds.has(project.project_id)"
                  @click.stop="toggleProjectSelection(project, $event)"
                  class="w-4 h-4 text-primary-600 rounded flex-shrink-0 cursor-pointer"
                />
                <div 
                  class="min-w-0 flex-1 cursor-pointer hover:text-primary-600 dark:hover:text-primary-400"
                  @click="toggleProjectSelection(project, $event)"
                >
                  <div class="flex items-center gap-2">
                    <h3 class="text-base font-semibold text-gray-900 dark:text-content-primary truncate">
                      {{ project.name }}
                    </h3>
                    <span
                      v-if="project.group"
                      @click.stop="openGroupDialog(project)"
                      class="badge badge-neutral hover:bg-gray-200 dark:hover:bg-surface-layer cursor-pointer"
                    >
                      {{ project.group }}
                    </span>
                  </div>
                  <p class="text-sm text-gray-500 dark:text-content-secondary mt-1 truncate" :title="project.path">
                    {{ project.path }}
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-1">
                <button
                  @click.stop="openGroupDialog(project)"
                  class="text-blue-600 hover:text-blue-800 p-1"
                  title="设置分组"
                >
                  <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
                  </svg>
                </button>
                <button
                  @click.stop="removeProject(project.project_id)"
                  class="text-red-600 hover:text-red-800 p-1"
                >
                  <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>
            <div class="mt-3 flex items-center justify-between text-sm">
              <span class="text-gray-600 dark:text-content-secondary">Godot {{ project.godot_version }}</span>
              <div class="flex items-center gap-2">
                <button
                  v-if="project.status === 'MissingSource'"
                  @click.stop="openRelocateDialog(project)"
                  class="px-2 py-1 rounded text-xs font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors"
                  title="重新定位项目"
                >
                  重新定位
                </button>
                <button
                  v-else
                  @click.stop="launchProject(project)"
                  :disabled="isLaunching || engines.length === 0"
                  class="px-2 py-1 rounded text-xs font-medium bg-primary-600 text-white hover:bg-primary-700 disabled:opacity-50 transition-colors"
                  title="启动项目"
                >
                  启动
                </button>
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
                  {{ project.status === 'Ready' ? '就绪' : project.status === 'Warning' ? '警告' : project.status === 'Conflict' ? '冲突' : project.status === 'MissingSource' ? '源缺失' : '错误' }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showProjectDetail && selectedProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showProjectDetail = false; selectedProject = null">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-lg shadow-xl" @click.stop>
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
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">项目路径</h4>
          <p class="text-sm text-gray-600 dark:text-gray-400 break-all bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
            {{ selectedProject.path }}
          </p>
        </div>
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">状态</h4>
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
            {{ selectedProject.status === 'Ready' ? '就绪' : selectedProject.status === 'Warning' ? '警告' : selectedProject.status === 'Conflict' ? '冲突' : selectedProject.status === 'MissingSource' ? '源缺失' : '错误' }}
          </span>
        </div>
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">引擎绑定</h4>
          <div class="flex items-center gap-2">
            <button
              @click="openEngineDialog(selectedProject)"
              class="px-3 py-1 rounded text-sm font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors"
            >
              {{ projectEngineBinding ? '更换引擎' : '绑定引擎' }}
            </button>
            <span v-if="projectEngineBinding" class="text-sm text-gray-600 dark:text-gray-400">
              已绑定
            </span>
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <button
            @click="showProjectDetail = false; selectedProject = null"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

    <div v-if="showScanDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showScanDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">选择扫描目录</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          选择一个目录，将递归扫描其中所有 Godot 项目（包含 project.godot 的目录）
        </p>
        <div class="flex gap-2 mb-6">
          <input
            v-model="scanDirInput"
            type="text"
            placeholder="请选择或输入目录路径"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          />
          <button
            @click="selectScanDir"
            class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
          >
            浏览
          </button>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            @click="showScanDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="startScan"
            :disabled="!scanDirInput"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            开始扫描
          </button>
        </div>
      </div>
    </div>

    <div v-if="showGroupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showGroupDialog = false; groupInput = ''; editingProjectId = null">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">设置项目分组</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          输入分组名称，相同分组的项目会显示在一起
        </p>
        <input
          v-model="groupInput"
          type="text"
          placeholder="输入分组名称（留空移除分组）"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
        />
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showGroupDialog = false; groupInput = ''; editingProjectId = null"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="saveGroup"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            保存
          </button>
        </div>
      </div>
    </div>

    <div v-if="showEngineDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showEngineDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          {{ selectedProject?.name }} - 引擎绑定
        </h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">选择引擎</label>
            <select
              v-model="selectedEngineId"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            >
              <option value="">请选择引擎</option>
              <option v-for="engine in engines" :key="engine.engine_id" :value="engine.engine_id">
                {{ engine.name }} (v{{ engine.version }}) {{ engine.is_default ? '- 默认' : '' }}
              </option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">启动参数（可选）</label>
            <input
              v-model="customArgs"
              type="text"
              placeholder="例如: --editor --quit"
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
            解除绑定
          </button>
          <div class="flex gap-2 ml-auto">
            <button
              @click="showEngineDialog = false"
              class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
            >
              取消
            </button>
            <button
              @click="bindEngine"
              :disabled="!selectedEngineId"
              class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
            >
              绑定
            </button>
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showConfirmDialog"
      :title="confirmAction?.title || ''"
      :description="confirmAction?.message || ''"
      confirm-text="确认删除"
      @confirm="onConfirmDialogConfirm"
    />

    <div v-if="showRelocateDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRelocateDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">重新定位项目</h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
          项目路径已失效，请选择新的项目目录。
        </p>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">新路径</label>
          <div class="flex gap-2">
            <input
              v-model="relocateNewPath"
              type="text"
              readonly
              placeholder="请选择项目目录"
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
            <button
              @click="selectRelocatePath"
              class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
            >
              浏览
            </button>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showRelocateDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="confirmRelocate"
            :disabled="!relocateNewPath"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            确认
          </button>
        </div>
      </div>
    </div>

    <div v-if="showMovedDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showMovedDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-lg shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">检测到项目迁移</h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
          以下项目的路径已失效，但发现了同名项目。是否更新路径？
        </p>
        <div class="space-y-3 max-h-60 overflow-y-auto">
          <div v-for="candidate in movedCandidates" :key="candidate.project_id" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-gray-900 dark:text-gray-100">{{ candidate.old_name }}</h4>
                <p class="text-xs text-red-500 dark:text-red-400 mt-1">旧路径: {{ candidate.old_path }}</p>
                <p class="text-xs text-green-500 dark:text-green-400">新路径: {{ candidate.new_path }}</p>
              </div>
              <div class="flex gap-2">
                <button
                  @click="confirmMovedProject(candidate)"
                  class="px-3 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 text-sm"
                >
                  更新
                </button>
                <button
                  @click="dismissMovedProject(candidate)"
                  class="px-3 py-1 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded hover:bg-gray-300 dark:hover:bg-gray-500 text-sm"
                >
                  忽略
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
            关闭
          </button>
        </div>
      </div>
    </div>
  </div>
</template>