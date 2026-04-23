<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'
import type { Project } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'

const projects = ref<Project[]>([])
const isLoading = ref(false)
const debugLog = ref<string[]>([])

onMounted(() => {
  loadProjects()
})

const addDebugLog = (message: string) => {
  const timestamp = new Date().toLocaleTimeString()
  debugLog.value.push(`[${timestamp}] ${message}`)
  console.log(message)
}

const loadProjects = async () => {
  isLoading.value = true
  addDebugLog('开始加载项目列表...')
  try {
    const result = await api.getProjects()
    projects.value = result
    addDebugLog(`成功加载 ${result.length} 个项目`)
  } catch (error) {
    addDebugLog(`加载项目失败: ${error}`)
    console.error('加载项目失败:', error)
  } finally {
    isLoading.value = false
  }
}

const scanProjects = async () => {
  isLoading.value = true
  addDebugLog('开始扫描项目...')
  try {
    const settings = await api.getSettings()
    const rootDirs = settings.scan_directories || ['D:\\']
    const result = await api.scanProjects(rootDirs)
    projects.value = result
    addDebugLog(`扫描完成，发现 ${result.length} 个项目`)
  } catch (error) {
    addDebugLog(`扫描项目失败: ${error}`)
    console.error('扫描项目失败:', error)
  } finally {
    isLoading.value = false
  }
}

const addProject = async () => {
  isLoading.value = true
  addDebugLog('开始添加项目...')
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Godot 项目目录'
    })

    if (selected && typeof selected === 'string') {
      const result = await api.addProject(selected)
      addDebugLog(`成功添加项目: ${result.name}`)
      await loadProjects()
    }
  } catch (error) {
    addDebugLog(`添加项目失败: ${error}`)
    console.error('添加项目失败:', error)
  } finally {
    isLoading.value = false
  }
}

const removeProject = async (project_id: string) => {
  addDebugLog(`删除项目: ${project_id}`)
  try {
    await api.removeProject(project_id)
    addDebugLog('项目删除成功')
    await loadProjects()
  } catch (error) {
    addDebugLog(`删除项目失败: ${error}`)
    console.error('删除项目失败:', error)
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">项目管理</h1>
      <div class="space-x-3">
        <button
          @click="scanProjects"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50"
        >
          扫描项目
        </button>
        <button
          @click="addProject"
          :disabled="isLoading"
          class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50"
        >
          添加项目
        </button>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="projects.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">暂无项目</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        开始扫描或手动添加 Godot 项目
      </p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div
        v-for="project in projects"
        :key="project.project_id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-lg transition-shadow p-6"
      >
        <div class="flex items-start justify-between min-w-0">
          <div class="min-w-0 flex-1">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 truncate">
              {{ project.name }}
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 truncate">
              {{ project.path }}
            </p>
          </div>
          <button
            @click="removeProject(project.project_id)"
            class="text-red-600 hover:text-red-800"
          >
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="mt-4 flex items-center text-sm text-gray-600 dark:text-gray-400">
          <span class="mr-4">Godot {{ project.godot_version }}</span>
        </div>
      </div>
    </div>

    <div v-if="debugLog.length > 0" class="mt-8">
      <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">调试日志</h3>
      <div class="bg-gray-100 dark:bg-gray-900 rounded-lg p-4 max-h-64 overflow-y-auto">
        <div v-for="(log, index) in debugLog" :key="index" class="text-sm text-gray-700 dark:text-gray-300 font-mono">
          {{ log }}
        </div>
      </div>
    </div>
  </div>
</template>
