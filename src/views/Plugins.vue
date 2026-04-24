<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'
import type { Plugin } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const plugins = ref<Plugin[]>([])
const isLoading = ref(false)
const gitUrl = ref('')
const showGitDialog = ref(false)

onMounted(() => {
  loadPlugins()
})

const loadPlugins = async () => {
  isLoading.value = true
  try {
    const result = await api.getPlugins()
    plugins.value = result
  } catch (error) {
    toast.error(`加载插件列表失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const importFromLocal = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Godot 插件目录'
    })
    if (selected && typeof selected === 'string') {
      isLoading.value = true
      const result = await api.importPluginFromLocal(selected)
      toast.success(`成功导入插件: ${result.name}`)
      await loadPlugins()
    }
  } catch (error) {
    toast.error(`导入插件失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const importFromFile = async () => {
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      title: '选择插件配置文件 (plugin.cfg)',
      filters: [{ name: 'Plugin Config', extensions: ['cfg'] }]
    })
    if (selected && typeof selected === 'string') {
      isLoading.value = true
      const dirPath = selected.substring(0, selected.lastIndexOf(/[/\\]/.test(selected) ? (selected.includes('\\') ? '\\' : '/') : '/'))
      const result = await api.importPluginFromLocal(dirPath || selected)
      toast.success(`成功导入插件: ${result.name}`)
      await loadPlugins()
    }
  } catch (error) {
    toast.error(`导入插件失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const importFromGit = async () => {
  if (!gitUrl.value) {
    toast.warning('请输入 Git URL')
    return
  }
  isLoading.value = true
  try {
    const result = await api.importPluginFromGit(gitUrl.value)
    toast.success(`成功导入插件: ${result.name}`)
    gitUrl.value = ''
    showGitDialog.value = false
    await loadPlugins()
  } catch (error) {
    toast.error(`从 Git 导入插件失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const removePlugin = async (plugin_id: string) => {
  try {
    await api.removePlugin(plugin_id)
    toast.success('插件已删除')
    await loadPlugins()
  } catch (error) {
    toast.error(`删除插件失败: ${error}`)
  }
}

const importFromProjects = async () => {
  isLoading.value = true
  try {
    const importedPlugins = await api.importPluginsFromProjects()
    if (importedPlugins.length > 0) {
      toast.success(`成功导入 ${importedPlugins.length} 个插件`)
    } else {
      toast.info('没有发现新的插件可以导入')
    }
    await loadPlugins()
  } catch (error) {
    toast.error(`从项目导入插件失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">插件仓库</h1>
      <div class="space-x-3">
        <button
          @click="importFromProjects"
          :disabled="isLoading"
          class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors disabled:opacity-50"
        >
          从项目导入
        </button>
        <button
          @click="importFromLocal"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50"
        >
          从目录导入
        </button>
        <button
          @click="importFromFile"
          :disabled="isLoading"
          class="px-4 py-2 bg-teal-600 text-white rounded-lg hover:bg-teal-700 transition-colors disabled:opacity-50"
        >
          从文件导入
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
        从本地目录、文件或 Git 仓库导入插件
      </p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div
        v-for="plugin in plugins"
        :key="plugin.plugin_id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-lg transition-shadow p-6"
      >
        <div class="flex items-start justify-between min-w-0">
          <div class="min-w-0 flex-1">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 truncate">
              {{ plugin.name }}
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
              {{ plugin.description || '无描述' }}
            </p>
          </div>
          <button
            @click="removePlugin(plugin.plugin_id)"
            class="text-red-600 hover:text-red-800 ml-2"
          >
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="mt-4 flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
          <span>v{{ plugin.versions[0]?.version || '1.0.0' }}</span>
          <span>{{ plugin.author || '未知作者' }}</span>
        </div>
        <div class="mt-2 flex items-center gap-2">
          <span
            :class="[
              'px-2 py-0.5 rounded text-xs font-medium',
              plugin.compatibility === 'Godot4' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400' :
              plugin.compatibility === 'Godot3' ? 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400' :
              plugin.compatibility === 'Both' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' :
              'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400'
            ]"
          >
            {{ plugin.compatibility === 'Godot4' ? 'Godot 4' : plugin.compatibility === 'Godot3' ? 'Godot 3' : plugin.compatibility === 'Both' ? '通用' : '未知' }}
          </span>
          <span class="px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400">
            {{ plugin.source.source_type }}
          </span>
        </div>
      </div>
    </div>

    <div v-if="showGitDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">从 Git 导入</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          输入 Git 仓库 URL，将克隆并导入其中的 Godot 插件
        </p>
        <input
          v-model="gitUrl"
          type="text"
          placeholder="https://github.com/user/plugin-repo.git"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
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
            :disabled="isLoading || !gitUrl"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            导入
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
