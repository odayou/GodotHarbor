<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { api, withErrorLogging } from '@/api'
import type { Project, Plugin, ProjectBinding, ApplyResult } from '@/types'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const projects = ref<Project[]>([])
const plugins = ref<Plugin[]>([])
const bindings = ref<ProjectBinding[]>([])
const selectedProjectId = ref<string | null>(null)
const isLoading = ref(false)
const showApplyDialog = ref(false)
const applyResult = ref<ApplyResult | null>(null)
const isApplying = ref(false)

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
    await loadBindings(newId)
  }
})

const bindPluginToProject = async (plugin_id: string) => {
  if (!selectedProjectId.value) return
  const plugin = plugins.value.find(p => p.plugin_id === plugin_id)
  if (!plugin || !plugin.versions.length) {
    toast.warning('该插件没有可用版本')
    return
  }
  const version = plugin.versions[0]
  const unit = version.units[0]
  if (!unit) {
    toast.warning('该插件版本没有可用的单元')
    return
  }
  const mountPath = `addons/${unit.name}`
  isLoading.value = true
  try {
    await withErrorLogging('Linker.bindPlugin', () =>
      api.bindPlugin(
        selectedProjectId.value!,
        plugin_id,
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
      <button
        @click="confirmApply"
        :disabled="isLoading || !selectedProjectId || projectBindings.length === 0"
        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
      >
        应用变更
      </button>
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
    </div>

    <div v-else class="grid grid-cols-1 lg:grid-cols-12 gap-4 lg:gap-6">
      <div class="lg:col-span-3 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">项目列表</h3>
        <div class="space-y-1 max-h-64 lg:max-h-none overflow-y-auto">
          <button
            v-for="project in projects"
            :key="project.project_id"
            @click="selectedProjectId = project.project_id"
            :class="[
              'w-full text-left px-3 py-2 rounded-lg transition-colors text-sm',
              selectedProjectId === project.project_id
                ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
            ]"
          >
            <div class="font-medium truncate">{{ project.name }}</div>
            <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Godot {{ project.godot_version }}</div>
          </button>
        </div>
      </div>

      <div class="lg:col-span-5 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">可用插件</h3>
        <div v-if="!selectedProjectId" class="text-center py-8 text-sm text-gray-500 dark:text-gray-400">
          请先选择一个项目
        </div>
        <div v-else-if="availablePlugins.length === 0" class="text-center py-8 text-sm text-gray-500 dark:text-gray-400">
          所有插件已绑定或暂无插件
        </div>
        <div v-else class="space-y-2 max-h-64 lg:max-h-96 overflow-y-auto">
          <div
            v-for="plugin in availablePlugins"
            :key="plugin.plugin_id"
            class="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg"
          >
            <div class="min-w-0 flex-1">
              <h4 class="font-medium text-gray-900 dark:text-gray-100 text-sm truncate">{{ plugin.name }}</h4>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                v{{ plugin.versions[0]?.version || '1.0.0' }} · {{ plugin.author || '未知' }}
              </p>
            </div>
            <button
              @click="bindPluginToProject(plugin.plugin_id)"
              :disabled="isLoading"
              class="ml-2 px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 disabled:opacity-50 whitespace-nowrap"
            >
              绑定
            </button>
          </div>
        </div>
      </div>

      <div class="lg:col-span-4 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">
          已绑定插件
          <span v-if="projectBindings.length > 0" class="text-xs font-normal text-gray-500 dark:text-gray-400 ml-1">
            ({{ projectBindings.length }})
          </span>
        </h3>
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
            class="flex items-center justify-between p-3 border border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/10 rounded-lg"
          >
            <div class="min-w-0 flex-1">
              <h4 class="font-medium text-gray-900 dark:text-gray-100 text-sm truncate">
                {{ item.plugin?.name || '未知插件' }}
              </h4>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                → {{ item.binding.mount_path }}
              </p>
            </div>
            <button
              @click="unbindPluginFromProject(item.binding.plugin_id)"
              :disabled="isLoading"
              class="ml-2 text-red-600 hover:text-red-800 text-xs whitespace-nowrap"
            >
              解绑
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showApplyDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-4 lg:p-6 w-full max-w-lg shadow-xl">
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
  </div>
</template>