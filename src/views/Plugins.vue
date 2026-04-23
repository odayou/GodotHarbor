<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'
import type { Plugin } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'

const plugins = ref<Plugin[]>([])
const isLoading = ref(false)
const gitUrl = ref('')
const showGitDialog = ref(false)
const debugLog = ref<string[]>([])

onMounted(() => {
  loadPlugins()
})

const addDebugLog = (message: string) => {
  const timestamp = new Date().toLocaleTimeString()
  debugLog.value.push(`[${timestamp}] ${message}`)
  console.log(message)
}

const loadPlugins = async () => {
  isLoading.value = true
  addDebugLog('开始加载插件列表...')
  try {
    const result = await api.getPlugins()
    plugins.value = result
    addDebugLog(`成功加载 ${result.length} 个插件`)
  } catch (error) {
    addDebugLog(`加载插件失败: ${error}`)
    console.error('加载插件失败:', error)
  } finally {
    isLoading.value = false
  }
}

const importFromLocal = async () => {
  isLoading.value = true
  addDebugLog('开始从本地导入插件...')
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Godot 插件目录'
    })

    if (selected && typeof selected === 'string') {
      const result = await api.importPluginFromLocal(selected)
      addDebugLog(`成功导入插件: ${result.name}`)
      await loadPlugins()
    }
  } catch (error) {
    addDebugLog(`导入插件失败: ${error}`)
    console.error('导入插件失败:', error)
  } finally {
    isLoading.value = false
  }
}

const importFromGit = async () => {
  if (!gitUrl.value) {
    addDebugLog('请输入 Git URL')
    return
  }
  
  isLoading.value = true
  addDebugLog(`开始从 Git 导入插件: ${gitUrl.value}`)
  try {
    const result = await api.importPluginFromGit(gitUrl.value)
    addDebugLog(`成功导入插件: ${result.name}`)
    gitUrl.value = ''
    showGitDialog.value = false
    await loadPlugins()
  } catch (error) {
    addDebugLog(`从 Git 导入插件失败: ${error}`)
    console.error('从 Git 导入插件失败:', error)
  } finally {
    isLoading.value = false
  }
}

const removePlugin = async (pluginId: string) => {
  addDebugLog(`删除插件: ${pluginId}`)
  try {
    await api.removePlugin(pluginId)
    addDebugLog('插件删除成功')
    await loadPlugins()
  } catch (error) {
    addDebugLog(`删除插件失败: ${error}`)
    console.error('删除插件失败:', error)
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">插件仓库</h1>
      <div class="space-x-3">
        <button
          @click="importFromLocal"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50"
        >
          从本地导入
        </button>
        <button
          @click="showGitDialog = true"
          :disabled="isLoading"
          class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50"
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
        :key="plugin.pluginId"
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
          <button
            @click="removePlugin(plugin.pluginId)"
            class="text-red-600 hover:text-red-800"
          >
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="mt-4 flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
          <span>v{{ plugin.versions[0]?.version || '1.0.0' }}</span>
          <span>{{ plugin.author }}</span>
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

    <div v-if="showGitDialog" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">从 Git 导入</h3>
        <input
          v-model="gitUrl"
          type="text"
          placeholder="输入 Git 仓库 URL"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
        />
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showGitDialog = false; gitUrl = ''"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="importFromGit"
            :disabled="isLoading"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            导入
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
