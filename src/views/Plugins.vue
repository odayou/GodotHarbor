<script setup lang="ts">
import { ref, onMounted } from 'vue'

const plugins = ref<any[]>([])
const isLoading = ref(false)

onMounted(() => {
  loadPlugins()
})

const loadPlugins = async () => {
  isLoading.value = true
  try {
    // TODO: 从后端加载插件列表
  } catch (error) {
    console.error('加载插件失败:', error)
  } finally {
    isLoading.value = false
  }
}

const importFromLocal = async () => {
  // TODO: 实现从本地导入
}

const importFromGit = async () => {
  // TODO: 实现从 Git 导入
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">插件仓库</h1>
      <div class="space-x-3">
        <button
          @click="importFromLocal"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
        >
          从本地导入
        </button>
        <button
          @click="importFromGit"
          class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
        >
          从 Git 导入
        </button>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="plugins.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">暂无插件</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        从本地目录或 Git 仓库导入插件
      </p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div
        v-for="plugin in plugins"
        :key="plugin.id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-lg transition-shadow p-6"
      >
        <div class="flex items-start justify-between">
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {{ plugin.name }}
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
              {{ plugin.description }}
            </p>
          </div>
        </div>
        <div class="mt-4 flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
          <span>v{{ plugin.version }}</span>
          <span>{{ plugin.usedBy }} 个项目使用</span>
        </div>
      </div>
    </div>
  </div>
</template>
