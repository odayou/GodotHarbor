<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/api'
import type { Project } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const projects = ref<Project[]>([])
const isLoading = ref(false)
const showScanDialog = ref(false)
const scanDirInput = ref('')
const showProjectDetail = ref(false)
const selectedProject = ref<Project | null>(null)
const showGroupDialog = ref(false)
const groupInput = ref('')
const editingProjectId = ref<string | null>(null)

const searchQuery = ref('')
const filterGroup = ref<string>('all')
const filterStatus = ref<string>('all')
const availableGroups = ref<string[]>([])

onMounted(() => {
  loadProjects()
  loadGroups()
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

const showProjectDetails = (project: Project) => {
  selectedProject.value = project
  showProjectDetail.value = true
}

const loadProjects = async () => {
  isLoading.value = true
  try {
    const result = await api.getProjects()
    projects.value = result
    await loadGroups()
  } catch (error) {
    toast.error(`加载项目失败: ${error}`)
  } finally {
    isLoading.value = false
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

const removeProject = async (projectId: string) => {
  try {
    await api.removeProject(projectId)
    toast.success('项目已删除')
    await loadProjects()
  } catch (error) {
    toast.error(`删除项目失败: ${error}`)
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
    toast.success('分组已更新')
    showGroupDialog.value = false
    editingProjectId.value = null
    groupInput.value = ''
    await loadGroups()
  } catch (error) {
    toast.error(`更新分组失败: ${error}`)
  }
}
</script>

<template>
  <div class="space-y-6">
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
          class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors disabled:opacity-50 text-sm"
        >
          快速扫描
        </button>
        <button
          @click="addProject"
          :disabled="isLoading"
          class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 text-sm"
        >
          添加项目
        </button>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
      <div class="flex flex-col lg:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索项目名称或路径..."
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          />
        </div>
        <div class="flex flex-wrap gap-2 items-center">
          <select
            v-model="filterGroup"
            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          >
            <option value="all">全部分组</option>
            <option value="ungrouped">未分组</option>
            <option v-for="group in availableGroups" :key="group" :value="group">{{ group }}</option>
          </select>
          <select
            v-model="filterStatus"
            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          >
            <option value="all">全部状态</option>
            <option value="Ready">就绪</option>
            <option value="Warning">警告</option>
            <option value="Error">错误</option>
          </select>
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="filteredProjects.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">暂无项目</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        开始扫描或手动添加 Godot 项目
      </p>
    </div>

    <div v-else class="space-y-6">
      <div v-for="(groupProjects, groupName) in (filterGroup === 'all' ? groupedProjects : { all: filteredProjects })" :key="groupName" class="space-y-3">
        <div v-if="filterGroup === 'all' && Object.keys(groupedProjects).length > 1" class="flex items-center gap-2">
          <h2 class="text-lg font-semibold text-gray-700 dark:text-gray-300">
            {{ groupName === '未分组' ? '未分组' : groupName }}
          </h2>
          <span class="text-sm text-gray-500 dark:text-gray-400">({{ groupProjects.length }} 个项目)</span>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <div
            v-for="project in (filterGroup === 'all' ? groupProjects : filteredProjects)"
            :key="project.project_id"
            class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-lg transition-shadow p-4"
          >
            <div class="flex items-start justify-between min-w-0">
              <div 
                class="flex items-center gap-3 min-w-0 flex-1 cursor-pointer hover:text-primary-600 dark:hover:text-primary-400"
                @click="showProjectDetails(project)"
              >
                <div class="flex-shrink-0 w-10 h-10 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700 flex items-center justify-center">
                  <img
                    v-if="project.icon_path"
                    :src="getIconUrl(project.icon_path)"
                    :alt="project.name"
                    class="w-10 h-10 object-contain"
                    @error="($event.target as HTMLImageElement).style.display = 'none'"
                  />
                  <svg v-else class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 truncate">
                      {{ project.name }}
                    </h3>
                    <button
                      v-if="project.group"
                      @click.stop="openGroupDialog(project)"
                      class="px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400 hover:bg-blue-200 dark:hover:bg-blue-800/50"
                    >
                      {{ project.group }}
                    </button>
                  </div>
                  <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 truncate" :title="project.path">
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
              <span class="text-gray-600 dark:text-gray-400">Godot {{ project.godot_version }}</span>
              <span
                :class="[
                  'px-2 py-0.5 rounded text-xs font-medium',
                  project.status === 'Ready' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' :
                  project.status === 'Warning' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400' :
                  'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
                ]"
              >
                {{ project.status === 'Ready' ? '就绪' : project.status === 'Warning' ? '警告' : '错误' }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showProjectDetail && selectedProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-lg shadow-xl">
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
              'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
            ]"
          >
            {{ selectedProject.status === 'Ready' ? '就绪' : selectedProject.status === 'Warning' ? '警告' : '错误' }}
          </span>
        </div>
        <div class="flex justify-end">
          <button
            @click="showProjectDetail = false; selectedProject = null"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

    <div v-if="showScanDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl">
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

    <div v-if="showGroupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl">
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
  </div>
</template>