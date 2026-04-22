<script setup lang="ts">
import { ref, onMounted } from 'vue'

const projects = ref<any[]>([])
const isLoading = ref(false)

onMounted(() => {
  loadProjects()
})

const loadProjects = async () => {
  isLoading.value = true
  try {
    // TODO: 从后端加载项目列表
  } catch (error) {
    console.error('加载项目失败:', error)
  } finally {
    isLoading.value = false
  }
}

const scanProjects = async () => {
  // TODO: 实现项目扫描
}

const addProject = async () => {
  // TODO: 实现添加项目
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">项目管理</h1>
      <div class="space-x-3">
        <button
          @click="scanProjects"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
        >
          扫描项目
        </button>
        <button
          @click="addProject"
          class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
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
        :key="project.id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-lg transition-shadow p-6"
      >
        <div class="flex items-start justify-between">
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {{ project.name }}
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
              {{ project.path }}
            </p>
          </div>
          <span
            :class="[
              'px-2 py-1 text-xs font-medium rounded-full',
              project.status === 'ready' ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' :
              project.status === 'warning' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200' :
              'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
            ]"
          >
            {{ project.status }}
          </span>
        </div>
        <div class="mt-4 flex items-center text-sm text-gray-600 dark:text-gray-400">
          <span class="mr-4">Godot {{ project.godotVersion }}</span>
          <span>{{ project.pluginCount }} 个插件</span>
        </div>
      </div>
    </div>
  </div>
</template>
