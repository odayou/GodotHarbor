<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { api, withErrorLogging } from '@/api'
import type { Project, Plugin, PluginVersion, PluginUnit, ProjectBinding, ApplyResult, BatchApplyResult, BatchBindingRequest } from '@/types'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'

const router = useRouter()
const toast = useToast()
const projects = ref<Project[]>([])
const plugins = ref<Plugin[]>([])
const bindings = ref<ProjectBinding[]>([])
const selectedProjectId = ref<string | null>(null)
const isLoading = ref(false)
const showApplyDialog = ref(false)
const applyResult = ref<ApplyResult | null>(null)
const isApplying = ref(false)

const selectedProjectIds = ref<Set<string>>(new Set())
const selectedAvailablePluginIds = ref<Set<string>>(new Set())
const selectedBoundPluginIds = ref<Set<string>>(new Set())

const lastClickedProjectIdx = ref(-1)
const lastClickedAvailablePluginIdx = ref(-1)
const lastClickedBoundPluginIdx = ref(-1)

const selectedProject = computed(() =>
  projects.value.find(p => p.project_id === selectedProjectId.value)
)

const projectBindings = computed(() =>
  bindings.value.filter(b => b.project_id === selectedProjectId.value)
)

const boundPluginIds = computed(() =>
  new Set(projectBindings.value.map(b => b.plugin_id))
)

const availablePlugins = computed(() =>
  plugins.value.filter(p => !boundPluginIds.value.has(p.plugin_id))
)

const boundPlugins = computed(() =>
  projectBindings.value.map(b => {
    const plugin = plugins.value.find(p => p.plugin_id === b.plugin_id)
    const version = plugin?.versions.find(v => v.version_id === b.version_id)
    return { binding: b, plugin, version }
  })
)

const selectedProjectCount = computed(() => selectedProjectIds.value.size)
const selectedAvailablePluginCount = computed(() => selectedAvailablePluginIds.value.size)
const selectedBoundPluginCount = computed(() => selectedBoundPluginIds.value.size)

const toggleProjectSelection = (project: Project, event: MouseEvent | Event) => {
  const mouseEvent = event as MouseEvent
  const projectId = project.project_id
  const idx = projects.value.findIndex(p => p.project_id === projectId)

  if (mouseEvent.shiftKey && lastClickedProjectIdx.value >= 0) {
    const start = Math.min(lastClickedProjectIdx.value, idx)
    const end = Math.max(lastClickedProjectIdx.value, idx)
    for (let i = start; i <= end; i++) {
      selectedProjectIds.value.add(projects.value[i].project_id)
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
    } else {
      selectedProjectIds.value.add(projectId)
    }
  }

  lastClickedProjectIdx.value = idx
  selectedProjectIds.value = new Set(selectedProjectIds.value)

  if (selectedProjectIds.value.size === 1) {
    selectedProjectId.value = Array.from(selectedProjectIds.value)[0]
  }
}

const toggleAvailablePluginSelection = (plugin: Plugin, event: MouseEvent | Event) => {
  const mouseEvent = event as MouseEvent
  const pluginId = plugin.plugin_id
  const idx = availablePlugins.value.findIndex(p => p.plugin_id === pluginId)

  if (mouseEvent.shiftKey && lastClickedAvailablePluginIdx.value >= 0) {
    const start = Math.min(lastClickedAvailablePluginIdx.value, idx)
    const end = Math.max(lastClickedAvailablePluginIdx.value, idx)
    for (let i = start; i <= end; i++) {
      selectedAvailablePluginIds.value.add(availablePlugins.value[i].plugin_id)
    }
  } else if (mouseEvent.ctrlKey || mouseEvent.metaKey) {
    if (selectedAvailablePluginIds.value.has(pluginId)) {
      selectedAvailablePluginIds.value.delete(pluginId)
    } else {
      selectedAvailablePluginIds.value.add(pluginId)
    }
  } else {
    if (selectedAvailablePluginIds.value.has(pluginId)) {
      selectedAvailablePluginIds.value.delete(pluginId)
    } else {
      selectedAvailablePluginIds.value.add(pluginId)
    }
  }

  lastClickedAvailablePluginIdx.value = idx
  selectedAvailablePluginIds.value = new Set(selectedAvailablePluginIds.value)
}

const toggleBoundPluginSelection = (pluginId: string, event: MouseEvent | Event) => {
  const mouseEvent = event as MouseEvent
  const idx = boundPlugins.value.findIndex(b => b.binding.plugin_id === pluginId)

  if (mouseEvent.shiftKey && lastClickedBoundPluginIdx.value >= 0) {
    const start = Math.min(lastClickedBoundPluginIdx.value, idx)
    const end = Math.max(lastClickedBoundPluginIdx.value, idx)
    for (let i = start; i <= end; i++) {
      selectedBoundPluginIds.value.add(boundPlugins.value[i].binding.plugin_id)
    }
  } else if (mouseEvent.ctrlKey || mouseEvent.metaKey) {
    if (selectedBoundPluginIds.value.has(pluginId)) {
      selectedBoundPluginIds.value.delete(pluginId)
    } else {
      selectedBoundPluginIds.value.add(pluginId)
    }
  } else {
    if (selectedBoundPluginIds.value.has(pluginId)) {
      selectedBoundPluginIds.value.delete(pluginId)
    } else {
      selectedBoundPluginIds.value.add(pluginId)
    }
  }

  lastClickedBoundPluginIdx.value = idx
  selectedBoundPluginIds.value = new Set(selectedBoundPluginIds.value)
}

const selectAllProjects = () => {
  for (const p of projects.value) {
    selectedProjectIds.value.add(p.project_id)
  }
  selectedProjectIds.value = new Set(selectedProjectIds.value)
}

const clearProjectSelection = () => {
  selectedProjectIds.value.clear()
  selectedProjectIds.value = new Set(selectedProjectIds.value)
  lastClickedProjectIdx.value = -1
}

const selectAllAvailablePlugins = () => {
  for (const p of availablePlugins.value) {
    selectedAvailablePluginIds.value.add(p.plugin_id)
  }
  selectedAvailablePluginIds.value = new Set(selectedAvailablePluginIds.value)
}

const clearAvailablePluginSelection = () => {
  selectedAvailablePluginIds.value.clear()
  selectedAvailablePluginIds.value = new Set(selectedAvailablePluginIds.value)
  lastClickedAvailablePluginIdx.value = -1
}

const selectAllBoundPlugins = () => {
  for (const item of boundPlugins.value) {
    selectedBoundPluginIds.value.add(item.binding.plugin_id)
  }
  selectedBoundPluginIds.value = new Set(selectedBoundPluginIds.value)
}

const clearBoundPluginSelection = () => {
  selectedBoundPluginIds.value.clear()
  selectedBoundPluginIds.value = new Set(selectedBoundPluginIds.value)
  lastClickedBoundPluginIdx.value = -1
}

const showBatchBindDialog = ref(false)
const isBatchBinding = ref(false)

const batchBindPlugins = async () => {
  const targetProjectIds = Array.from(selectedProjectIds.value)
  const targetPluginIds = Array.from(selectedAvailablePluginIds.value)

  if (targetProjectIds.length === 0) {
    toast.warning('请先选择至少一个项目')
    return
  }
  if (targetPluginIds.length === 0) {
    toast.warning('请先选择至少一个插件')
    return
  }

  showBatchBindDialog.value = true
}

const confirmBatchBind = async () => {
  const targetProjectIds = Array.from(selectedProjectIds.value)
  const targetPluginIds = Array.from(selectedAvailablePluginIds.value)

  const batchBindings: BatchBindingRequest[] = []
  for (const projectId of targetProjectIds) {
    for (const pluginId of targetPluginIds) {
      const plugin = plugins.value.find(p => p.plugin_id === pluginId)
      if (!plugin || !plugin.versions.length) continue

      const version = plugin.versions[0]
      const unit = version.units[0]
      if (!unit) continue

      const existingBinding = bindings.value.find(b => b.project_id === projectId && b.plugin_id === pluginId)
      if (existingBinding) continue

      batchBindings.push({
        project_id: projectId,
        plugin_id: pluginId,
        version_id: version.version_id,
        unit_id: unit.unit_id,
        mount_path: `addons/${unit.name}`
      })
    }
  }

  if (batchBindings.length === 0) {
    toast.info('没有需要绑定的组合（所有选中项目已绑定选中插件）')
    showBatchBindDialog.value = false
    return
  }

  isBatchBinding.value = true
  try {
    const result = await api.batchBindPlugins(batchBindings)
    if (result.failed_count > 0) {
      toast.warning(`批量绑定完成: 成功 ${result.success_count} 个, 失败 ${result.failed_count} 个`)
    } else {
      toast.success(`已成功绑定 ${result.success_count} 个插件`)
    }
    clearAvailablePluginSelection()
    showBatchBindDialog.value = false
    if (selectedProjectId.value) {
      await loadBindings(selectedProjectId.value)
    }
  } catch (error) {
    toast.error(`批量绑定失败: ${error}`)
  } finally {
    isBatchBinding.value = false
  }
}

const showBatchUnbindDialog = ref(false)
const isBatchUnbinding = ref(false)

const batchUnbindPlugins = async () => {
  if (!selectedProjectId.value) {
    toast.warning('请先选择一个项目')
    return
  }
  if (selectedBoundPluginIds.value.size === 0) {
    toast.warning('请先选择要解绑的插件')
    return
  }
  showBatchUnbindDialog.value = true
}

const confirmBatchUnbind = async () => {
  if (!selectedProjectId.value) return
  const pluginIds = Array.from(selectedBoundPluginIds.value)

  isBatchUnbinding.value = true
  try {
    const result = await api.batchUnbindPlugins(selectedProjectId.value, pluginIds)
    if (result.failed_count > 0) {
      toast.warning(`批量解绑完成: 成功 ${result.success_count} 个, 失败 ${result.failed_count} 个`)
    } else {
      toast.success(`已成功解绑 ${result.success_count} 个插件`)
    }
    clearBoundPluginSelection()
    showBatchUnbindDialog.value = false
    await loadBindings(selectedProjectId.value)
  } catch (error) {
    toast.error(`批量解绑失败: ${error}`)
  } finally {
    isBatchUnbinding.value = false
  }
}

const showBatchApplyDialog = ref(false)
const batchApplyResult = ref<BatchApplyResult | null>(null)
const isBatchApplying = ref(false)

const batchApplyChanges = () => {
  const targetIds = selectedProjectIds.value.size > 0
    ? Array.from(selectedProjectIds.value)
    : (selectedProjectId.value ? [selectedProjectId.value] : [])

  if (targetIds.length === 0) {
    toast.warning('请先选择至少一个项目')
    return
  }

  const projectsWithBindings = targetIds.filter(id =>
    bindings.value.some(b => b.project_id === id)
  )

  if (projectsWithBindings.length === 0) {
    toast.warning('选中的项目没有绑定任何插件')
    return
  }

  showBatchApplyDialog.value = true
}

const confirmBatchApply = async () => {
  const targetIds = selectedProjectIds.value.size > 0
    ? Array.from(selectedProjectIds.value)
    : (selectedProjectId.value ? [selectedProjectId.value] : [])

  if (targetIds.length === 0) return

  isBatchApplying.value = true
  try {
    batchApplyResult.value = await api.batchApplyChanges(targetIds)
    const successCount = batchApplyResult.value.results.filter(r => r.success).length
    const failCount = batchApplyResult.value.results.filter(r => !r.success).length
    if (failCount > 0) {
      toast.warning(`批量应用完成: 成功 ${successCount} 个, 失败 ${failCount} 个`)
    } else {
      toast.success(`已成功应用 ${successCount} 个项目的变更`)
    }
  } catch (error) {
    toast.error(`批量应用变更失败: ${error}`)
  } finally {
    isBatchApplying.value = false
  }
}

const closeBatchApplyDialog = () => {
  showBatchApplyDialog.value = false
  batchApplyResult.value = null
}

onMounted(async () => {
  await loadData()
})

const loadData = async () => {
  isLoading.value = true
  try {
    const [projectList, pluginList] = await Promise.all([
      withErrorLogging('Linker.loadProjects', () => api.getProjects()),
      withErrorLogging('Linker.loadPlugins', () => api.getPlugins())
    ])
    projects.value = projectList
    plugins.value = pluginList
    if (projectList.length > 0 && !selectedProjectId.value) {
      selectedProjectId.value = projectList[0].project_id
    }
    if (selectedProjectId.value) {
      await loadBindings(selectedProjectId.value)
    }
  } catch (error) {
    toast.error(`加载数据失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const loadBindings = async (projectId: string) => {
  try {
    bindings.value = await withErrorLogging('Linker.loadBindings', () => api.getProjectBindings(projectId))
  } catch (error) {
    toast.error(`加载绑定关系失败: ${error}`)
  }
}

watch(selectedProjectId, async (newId) => {
  if (newId) {
    selectedBoundPluginIds.value.clear()
    selectedBoundPluginIds.value = new Set(selectedBoundPluginIds.value)
    await loadBindings(newId)
  }
})

const showGraphView = ref(false)

const graphNodes = computed(() => {
  const nodes: Array<{ id: string; label: string; type: string; x: number; y: number }> = []
  const projectList = projects.value
  const pluginsList = plugins.value

  const projectSpacing = Math.min(120, 600 / Math.max(projectList.length, 1))
  const pluginSpacing = Math.min(120, 600 / Math.max(pluginsList.length, 1))

  projectList.forEach((p: Project, i: number) => {
    nodes.push({
      id: p.project_id,
      label: p.name,
      type: 'project',
      x: 80,
      y: 40 + i * projectSpacing
    })
  })

  pluginsList.forEach((p: Plugin, i: number) => {
    nodes.push({
      id: p.plugin_id,
      label: p.name,
      type: 'plugin',
      x: 520,
      y: 40 + i * pluginSpacing
    })
  })

  return nodes
})

const graphEdges = computed(() => {
  return bindings.value.map(b => ({
    from: b.project_id,
    to: b.plugin_id,
    version: b.version_id
  }))
})

const getNodePos = (id: string) => {
  const node = graphNodes.value.find(n => n.id === id)
  return node ? { x: node.x, y: node.y } : { x: 0, y: 0 }
}

const showVersionDialog = ref(false)
const versionSelectPlugin = ref<Plugin | null>(null)
const selectedVersionIdx = ref(0)
const selectedUnitIdx = ref(0)

useDialogEscape(showApplyDialog)
useDialogEscape(showVersionDialog)
useDialogEscape(showBatchBindDialog)
useDialogEscape(showBatchUnbindDialog)
useDialogEscape(showBatchApplyDialog)

const openVersionSelect = (plugin: Plugin) => {
  versionSelectPlugin.value = plugin
  selectedVersionIdx.value = 0
  selectedUnitIdx.value = 0
  showVersionDialog.value = true
}

const confirmVersionSelect = async () => {
  if (!selectedProjectId.value || !versionSelectPlugin.value) return
  const plugin = versionSelectPlugin.value
  const version = plugin.versions[selectedVersionIdx.value]
  const unit = version?.units[selectedUnitIdx.value]
  if (!version || !unit) {
    toast.warning('该插件版本没有可用的单元')
    return
  }
  const mountPath = `addons/${unit.name}`
  isLoading.value = true
  showVersionDialog.value = false
  try {
    await withErrorLogging('Linker.bindPlugin', () =>
      api.bindPlugin(
        selectedProjectId.value!,
        plugin.plugin_id,
        version.version_id,
        unit.unit_id,
        mountPath
      )
    )
    toast.success(`已绑定插件: ${plugin.name} v${version.version}`)
    await loadBindings(selectedProjectId.value)
  } catch (error) {
    toast.error(`绑定插件失败: ${error}`)
  } finally {
    isLoading.value = false
    versionSelectPlugin.value = null
  }
}

const bindPluginToProject = (plugin_id: string) => {
  if (!selectedProjectId.value) return
  const plugin = plugins.value.find(p => p.plugin_id === plugin_id)
  if (!plugin || !plugin.versions.length) {
    toast.warning('该插件没有可用版本')
    return
  }
  if (plugin.versions.length === 1 && plugin.versions[0].units.length <= 1) {
    const version = plugin.versions[0]
    const unit = version.units[0]
    if (!unit) {
      toast.warning('该插件版本没有可用的单元')
      return
    }
    doBindPlugin(plugin, version, unit)
  } else {
    openVersionSelect(plugin)
  }
}

const doBindPlugin = async (plugin: Plugin, version: PluginVersion, unit: PluginUnit) => {
  if (!selectedProjectId.value) return
  const mountPath = `addons/${unit.name}`
  isLoading.value = true
  try {
    await withErrorLogging('Linker.bindPlugin', () =>
      api.bindPlugin(
        selectedProjectId.value!,
        plugin.plugin_id,
        version.version_id,
        unit.unit_id,
        mountPath
      )
    )
    toast.success(`已绑定插件: ${plugin.name}`)
    await loadBindings(selectedProjectId.value)
  } catch (error) {
    toast.error(`绑定插件失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const unbindPluginFromProject = async (plugin_id: string) => {
  if (!selectedProjectId.value) return
  const binding = bindings.value.find(b => b.project_id === selectedProjectId.value && b.plugin_id === plugin_id)
  if (!binding) return
  isLoading.value = true
  try {
    await withErrorLogging('Linker.unbindPlugin', () =>
      api.unbindPlugin(selectedProjectId.value!, plugin_id)
    )
    toast.success('已取消绑定')
    await loadBindings(selectedProjectId.value)
  } catch (error) {
    toast.error(`取消绑定失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const confirmApply = () => {
  if (projectBindings.value.length === 0) {
    toast.warning('当前项目没有绑定任何插件')
    return
  }
  showApplyDialog.value = true
}

const applyChanges = async () => {
  if (!selectedProjectId.value) return
  isApplying.value = true
  try {
    applyResult.value = await withErrorLogging('Linker.applyChanges', () =>
      api.applyChanges(selectedProjectId.value!)
    )
    if (applyResult.value.success) {
      toast.success('变更已成功应用')
    } else {
      toast.error(`应用变更时出现错误: ${applyResult.value.errors.join(', ')}`)
    }
  } catch (error) {
    toast.error(`应用变更失败: ${error}`)
  } finally {
    isApplying.value = false
  }
}

const closeApplyDialog = () => {
  showApplyDialog.value = false
  applyResult.value = null
}
</script>

<template>
  <div class="space-y-4 lg:space-y-6">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">插件绑定</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="showGraphView = !showGraphView"
          class="px-4 py-2 text-sm rounded-lg transition-colors"
          :class="showGraphView ? 'bg-primary-600 text-white' : 'bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-500'"
        >
          {{ showGraphView ? '列表视图' : '图形视图' }}
        </button>
        <button
          @click="batchApplyChanges"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
        >
          批量应用变更
        </button>
        <button
          @click="confirmApply"
          :disabled="isLoading || !selectedProjectId || projectBindings.length === 0"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          应用变更
        </button>
      </div>
    </div>

    <div v-if="isLoading && projects.length === 0" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="projects.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">暂无项目</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">请先在项目管理中添加项目</p>
      <div class="mt-4 flex justify-center">
        <button
          @click="router.push('/projects')"
          class="inline-flex items-center gap-1.5 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
          </svg>
          前往项目
        </button>
      </div>
    </div>

    <div v-else class="grid grid-cols-1 lg:grid-cols-12 gap-4 lg:gap-6">
      <div class="lg:col-span-3 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">项目列表</h3>
          <div v-if="selectedProjectCount > 0" class="flex items-center gap-2">
            <span class="text-xs text-primary-600 dark:text-primary-400">已选 {{ selectedProjectCount }}</span>
            <button @click="selectAllProjects" class="text-xs text-primary-600 dark:text-primary-400 hover:underline">全选</button>
            <button @click="clearProjectSelection" class="text-xs text-gray-500 dark:text-gray-400 hover:underline">清除</button>
          </div>
        </div>
        <div class="space-y-1 max-h-64 lg:max-h-none overflow-y-auto">
          <div
            v-for="project in projects"
            :key="project.project_id"
            @click="toggleProjectSelection(project, $event)"
            :class="[
              'w-full flex items-center gap-2 px-3 py-2 rounded-lg transition-colors text-sm cursor-pointer',
              selectedProjectIds.has(project.project_id)
                ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400 ring-1 ring-primary-300 dark:ring-primary-700'
                : selectedProjectId === project.project_id
                  ? 'bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                  : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
            ]"
          >
            <input
              type="checkbox"
              :checked="selectedProjectIds.has(project.project_id)"
              class="w-3.5 h-3.5 text-primary-600 rounded flex-shrink-0 cursor-pointer"
              @click.stop="toggleProjectSelection(project, $event)"
            />
            <div class="min-w-0 flex-1">
              <div class="font-medium truncate">{{ project.name }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Godot {{ project.godot_version }}</div>
            </div>
          </div>
        </div>
      </div>

      <div class="lg:col-span-5 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">可用插件</h3>
          <div class="flex items-center gap-2">
            <div v-if="selectedAvailablePluginCount > 0">
              <span class="text-xs text-primary-600 dark:text-primary-400">已选 {{ selectedAvailablePluginCount }}</span>
              <button @click="selectAllAvailablePlugins" class="text-xs text-primary-600 dark:text-primary-400 hover:underline ml-1">全选</button>
              <button @click="clearAvailablePluginSelection" class="text-xs text-gray-500 dark:text-gray-400 hover:underline ml-1">清除</button>
            </div>
            <button
              v-if="selectedProjectCount > 0 && selectedAvailablePluginCount > 0"
              @click="batchBindPlugins"
              :disabled="isLoading"
              class="px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 disabled:opacity-50 whitespace-nowrap"
            >
              批量绑定 ({{ selectedProjectCount }}项目 × {{ selectedAvailablePluginCount }}插件)
            </button>
          </div>
        </div>
        <div v-if="!selectedProjectId && selectedProjectCount === 0" class="text-center py-8 text-sm text-gray-500 dark:text-gray-400">
          请先选择一个项目
        </div>
        <div v-else-if="availablePlugins.length === 0" class="text-center py-8 text-sm text-gray-500 dark:text-gray-400">
          所有插件已绑定或暂无插件
        </div>
        <div v-else class="space-y-2 max-h-64 lg:max-h-96 overflow-y-auto">
          <div
            v-for="plugin in availablePlugins"
            :key="plugin.plugin_id"
            :class="[
              'flex items-center justify-between p-3 border rounded-lg cursor-pointer',
              selectedAvailablePluginIds.has(plugin.plugin_id)
                ? 'border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-900/10'
                : 'border-gray-200 dark:border-gray-700'
            ]"
            @click="toggleAvailablePluginSelection(plugin, $event)"
          >
            <div class="flex items-center gap-2 min-w-0 flex-1">
              <input
                type="checkbox"
                :checked="selectedAvailablePluginIds.has(plugin.plugin_id)"
                class="w-3.5 h-3.5 text-primary-600 rounded flex-shrink-0 cursor-pointer"
                @click.stop="toggleAvailablePluginSelection(plugin, $event)"
              />
              <div class="min-w-0 flex-1">
                <h4 class="font-medium text-gray-900 dark:text-gray-100 text-sm truncate">{{ plugin.name }}</h4>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                  v{{ plugin.versions[0]?.version || '1.0.0' }} · {{ plugin.author || '未知' }}
                </p>
              </div>
            </div>
            <button
              @click.stop="bindPluginToProject(plugin.plugin_id)"
              :disabled="isLoading || (!selectedProjectId && selectedProjectCount === 0)"
              class="ml-2 px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 disabled:opacity-50 whitespace-nowrap"
            >
              绑定
            </button>
          </div>
        </div>
      </div>

      <div class="lg:col-span-4 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            已绑定插件
            <span v-if="projectBindings.length > 0" class="text-xs font-normal text-gray-500 dark:text-gray-400 ml-1">
              ({{ projectBindings.length }})
            </span>
          </h3>
          <div class="flex items-center gap-2">
            <div v-if="selectedBoundPluginCount > 0">
              <span class="text-xs text-red-600 dark:text-red-400">已选 {{ selectedBoundPluginCount }}</span>
              <button @click="selectAllBoundPlugins" class="text-xs text-primary-600 dark:text-primary-400 hover:underline ml-1">全选</button>
              <button @click="clearBoundPluginSelection" class="text-xs text-gray-500 dark:text-gray-400 hover:underline ml-1">清除</button>
            </div>
            <button
              v-if="selectedBoundPluginCount > 0 && selectedProjectId"
              @click="batchUnbindPlugins"
              :disabled="isLoading"
              class="px-3 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 disabled:opacity-50 whitespace-nowrap"
            >
              批量解绑 ({{ selectedBoundPluginCount }})
            </button>
          </div>
        </div>
        <div v-if="!selectedProjectId" class="text-center py-8 text-sm text-gray-500 dark:text-gray-400">
          请先选择一个项目
        </div>
        <div v-else-if="boundPlugins.length === 0" class="text-center py-8 text-sm text-gray-500 dark:text-gray-400">
          尚未绑定任何插件
        </div>
        <div v-else class="space-y-2 max-h-64 lg:max-h-96 overflow-y-auto">
          <div
            v-for="item in boundPlugins"
            :key="item.binding.plugin_id"
            :class="[
              'flex items-center justify-between p-3 border rounded-lg cursor-pointer',
              selectedBoundPluginIds.has(item.binding.plugin_id)
                ? 'border-red-300 dark:border-red-700 bg-red-50 dark:bg-red-900/10'
                : 'border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/10'
            ]"
            @click="toggleBoundPluginSelection(item.binding.plugin_id, $event)"
          >
            <div class="flex items-center gap-2 min-w-0 flex-1">
              <input
                type="checkbox"
                :checked="selectedBoundPluginIds.has(item.binding.plugin_id)"
                class="w-3.5 h-3.5 text-red-600 rounded flex-shrink-0 cursor-pointer"
                @click.stop="toggleBoundPluginSelection(item.binding.plugin_id, $event)"
              />
              <div class="min-w-0 flex-1">
                <h4 class="font-medium text-gray-900 dark:text-gray-100 text-sm truncate">
                  {{ item.plugin?.name || '未知插件' }}
                </h4>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                  → {{ item.binding.mount_path }}
                </p>
              </div>
            </div>
            <button
              @click.stop="unbindPluginFromProject(item.binding.plugin_id)"
              :disabled="isLoading"
              class="ml-2 text-red-600 hover:text-red-800 text-xs whitespace-nowrap"
            >
              解绑
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showGraphView" class="bg-white dark:bg-gray-800 rounded-lg shadow p-4 mt-4">
      <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">绑定关系图</h3>
      <div class="overflow-auto">
        <svg :width="640" :height="Math.max(400, graphNodes.length * 30 + 80)" class="mx-auto">
          <defs>
            <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
              <polygon points="0 0, 8 3, 0 6" class="fill-primary-400" />
            </marker>
          </defs>
          <g v-for="edge in graphEdges" :key="edge.from + edge.to">
            <line
              :x1="getNodePos(edge.from).x + 80"
              :y1="getNodePos(edge.from).y + 12"
              :x2="getNodePos(edge.to).x - 4"
              :y2="getNodePos(edge.to).y + 12"
              stroke="currentColor"
              class="text-primary-300 dark:text-primary-600"
              stroke-width="2"
              marker-end="url(#arrowhead)"
            />
          </g>
          <g v-for="node in graphNodes" :key="node.id">
            <rect
              :x="node.x - 4"
              :y="node.y - 2"
              :width="node.type === 'project' ? 88 : 88"
              height="28"
              rx="6"
              :class="node.type === 'project'
                ? 'fill-blue-100 dark:fill-blue-900/30 stroke-blue-300 dark:stroke-blue-700'
                : 'fill-green-100 dark:fill-green-900/30 stroke-green-300 dark:stroke-green-700'"
              stroke-width="1.5"
            />
            <text
              :x="node.x + 40"
              :y="node.y + 16"
              text-anchor="middle"
              :class="node.type === 'project'
                ? 'fill-blue-800 dark:fill-blue-300'
                : 'fill-green-800 dark:fill-green-300'"
              font-size="11"
              font-weight="500"
            >
              {{ node.label.length > 10 ? node.label.substring(0, 10) + '…' : node.label }}
            </text>
          </g>
          <text x="80" y="20" text-anchor="middle" class="fill-gray-500 dark:fill-gray-400" font-size="11">项目</text>
          <text x="560" y="20" text-anchor="middle" class="fill-gray-500 dark:fill-gray-400" font-size="11">插件</text>
        </svg>
      </div>
    </div>

    <div v-if="showApplyDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click="closeApplyDialog">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-4 lg:p-6 w-full max-w-lg shadow-xl" @click.stop>
        <template v-if="!applyResult">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">确认应用变更</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
            将为项目 <strong class="text-gray-900 dark:text-gray-100">{{ selectedProject?.name }}</strong> 应用以下插件绑定：
          </p>
          <div class="space-y-2 mb-6 max-h-48 overflow-y-auto">
            <div
              v-for="item in boundPlugins"
              :key="item.binding.plugin_id"
              class="flex items-center gap-2 text-sm p-2 bg-gray-50 dark:bg-gray-700 rounded"
            >
              <svg class="h-4 w-4 text-green-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <span class="text-gray-900 dark:text-gray-100">{{ item.plugin?.name }}</span>
              <span class="text-gray-500 dark:text-gray-400">→</span>
              <span class="text-gray-600 dark:text-gray-300 truncate">{{ item.binding.mount_path }}</span>
            </div>
          </div>
          <div class="flex justify-end space-x-3">
            <button
              @click="closeApplyDialog"
              class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
            >
              取消
            </button>
            <button
              @click="applyChanges"
              :disabled="isApplying"
              class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
            >
              {{ isApplying ? '应用中...' : '确认应用' }}
            </button>
          </div>
        </template>
        <template v-else>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
            {{ applyResult.success ? '应用成功' : '应用完成（有错误）' }}
          </h3>
          <div v-if="applyResult.created.length > 0" class="mb-3">
            <p class="text-sm font-medium text-green-600 dark:text-green-400 mb-1">已创建：</p>
            <div v-for="path in applyResult.created" :key="path" class="text-xs text-gray-600 dark:text-gray-400 ml-3 break-all">
              {{ path }}
            </div>
          </div>
          <div v-if="applyResult.removed.length > 0" class="mb-3">
            <p class="text-sm font-medium text-yellow-600 dark:text-yellow-400 mb-1">已移除：</p>
            <div v-for="path in applyResult.removed" :key="path" class="text-xs text-gray-600 dark:text-gray-400 ml-3 break-all">
              {{ path }}
            </div>
          </div>
          <div v-if="applyResult.errors.length > 0" class="mb-3">
            <p class="text-sm font-medium text-red-600 dark:text-red-400 mb-1">错误：</p>
            <div v-for="err in applyResult.errors" :key="err" class="text-xs text-red-600 dark:text-red-400 ml-3">
              {{ err }}
            </div>
          </div>
          <div class="flex justify-end mt-4">
            <button
              @click="closeApplyDialog"
              class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
            >
              关闭
            </button>
          </div>
        </template>
      </div>
    </div>

    <div v-if="showBatchBindDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click="showBatchBindDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-4 lg:p-6 w-full max-w-lg shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">确认批量绑定</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          将为 <strong class="text-gray-900 dark:text-gray-100">{{ selectedProjectCount }} 个项目</strong> 绑定
          <strong class="text-gray-900 dark:text-gray-100">{{ selectedAvailablePluginCount }} 个插件</strong>：
        </p>
        <div class="space-y-3 mb-6 max-h-60 overflow-y-auto">
          <div>
            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">目标项目：</p>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="id in selectedProjectIds"
                :key="id"
                class="px-2 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 text-xs rounded"
              >
                {{ projects.find(p => p.project_id === id)?.name || id }}
              </span>
            </div>
          </div>
          <div>
            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">绑定插件：</p>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="id in selectedAvailablePluginIds"
                :key="id"
                class="px-2 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300 text-xs rounded"
              >
                {{ plugins.find(p => p.plugin_id === id)?.name || id }}
              </span>
            </div>
          </div>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            @click="showBatchBindDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="confirmBatchBind"
            :disabled="isBatchBinding"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ isBatchBinding ? '绑定中...' : '确认绑定' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showBatchUnbindDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click="showBatchUnbindDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-4 lg:p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">确认批量解绑</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          将从项目 <strong class="text-gray-900 dark:text-gray-100">{{ selectedProject?.name }}</strong> 解绑
          <strong class="text-red-600 dark:text-red-400">{{ selectedBoundPluginCount }} 个插件</strong>：
        </p>
        <div class="flex flex-wrap gap-1 mb-6">
          <span
            v-for="id in selectedBoundPluginIds"
            :key="id"
            class="px-2 py-0.5 bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300 text-xs rounded"
          >
            {{ plugins.find(p => p.plugin_id === id)?.name || id }}
          </span>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            @click="showBatchUnbindDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="confirmBatchUnbind"
            :disabled="isBatchUnbinding"
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50"
          >
            {{ isBatchUnbinding ? '解绑中...' : '确认解绑' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showBatchApplyDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click="closeBatchApplyDialog">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-4 lg:p-6 w-full max-w-lg shadow-xl" @click.stop>
        <template v-if="!batchApplyResult">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">确认批量应用变更</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
            将为以下项目应用所有插件绑定变更：
          </p>
          <div class="space-y-1 mb-6 max-h-48 overflow-y-auto">
            <div
              v-for="id in (selectedProjectIds.size > 0 ? selectedProjectIds : (selectedProjectId ? [selectedProjectId] : []))"
              :key="id"
              class="flex items-center gap-2 text-sm p-2 bg-gray-50 dark:bg-gray-700 rounded"
            >
              <svg class="h-4 w-4 text-primary-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              <span class="text-gray-900 dark:text-gray-100">{{ projects.find(p => p.project_id === id)?.name || id }}</span>
              <span class="text-gray-500 dark:text-gray-400 text-xs ml-auto">
                {{ bindings.filter(b => b.project_id === id).length }} 个绑定
              </span>
            </div>
          </div>
          <div class="flex justify-end space-x-3">
            <button
              @click="closeBatchApplyDialog"
              class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
            >
              取消
            </button>
            <button
              @click="confirmBatchApply"
              :disabled="isBatchApplying"
              class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
            >
              {{ isBatchApplying ? '应用中...' : '确认应用' }}
            </button>
          </div>
        </template>
        <template v-else>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">批量应用结果</h3>
          <div class="space-y-3 max-h-80 overflow-y-auto">
            <div
              v-for="result in batchApplyResult.results"
              :key="result.project_id"
              :class="[
                'p-3 rounded-lg border',
                result.success ? 'border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/10' : 'border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/10'
              ]"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-sm text-gray-900 dark:text-gray-100">{{ result.project_name }}</span>
                <span :class="['text-xs px-2 py-0.5 rounded', result.success ? 'bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-300' : 'bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-300']">
                  {{ result.success ? '成功' : '失败' }}
                </span>
              </div>
              <div v-if="result.created.length > 0" class="mt-1 text-xs text-green-600 dark:text-green-400">
                创建 {{ result.created.length }} 项
              </div>
              <div v-if="result.removed.length > 0" class="mt-0.5 text-xs text-yellow-600 dark:text-yellow-400">
                移除 {{ result.removed.length }} 项
              </div>
              <div v-if="result.errors.length > 0" class="mt-0.5 text-xs text-red-600 dark:text-red-400">
                错误: {{ result.errors.join(', ') }}
              </div>
            </div>
          </div>
          <div class="flex justify-end mt-4">
            <button
              @click="closeBatchApplyDialog"
              class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
            >
              关闭
            </button>
          </div>
        </template>
      </div>
    </div>

    <div v-if="showVersionDialog && versionSelectPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click="showVersionDialog = false; versionSelectPlugin = null">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-4 lg:p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          选择版本 - {{ versionSelectPlugin.name }}
        </h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">选择版本</label>
            <select
              v-model="selectedVersionIdx"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            >
              <option v-for="(ver, idx) in versionSelectPlugin.versions" :key="ver.version_id" :value="idx">
                v{{ ver.version }} ({{ new Date(ver.created_at).toLocaleDateString() }})
              </option>
            </select>
          </div>
          <div v-if="versionSelectPlugin.versions[selectedVersionIdx]?.units.length > 1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">选择单元</label>
            <select
              v-model="selectedUnitIdx"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            >
              <option v-for="(unit, idx) in versionSelectPlugin.versions[selectedVersionIdx]?.units" :key="unit.unit_id" :value="idx">
                {{ unit.name }}{{ unit.subdirectory ? ` (${unit.subdirectory})` : '' }}
              </option>
            </select>
          </div>
          <div class="text-xs text-gray-500 dark:text-gray-400">
            挂载路径: addons/{{ versionSelectPlugin.versions[selectedVersionIdx]?.units[selectedUnitIdx]?.name || '?' }}
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showVersionDialog = false; versionSelectPlugin = null"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="confirmVersionSelect"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            确认绑定
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
