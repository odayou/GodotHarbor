<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { api } from '@/api'
import type { Plugin } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/composables/useI18n'

const toast = useToast()
const { t } = useI18n()
const plugins = ref<Plugin[]>([])
const isLoading = ref(false)
const gitUrl = ref('')
const showGitDialog = ref(false)
const showPluginDetail = ref(false)
const selectedPlugin = ref<Plugin | null>(null)

const searchQuery = ref('')
const filterCompatibility = ref<string>('all')
const filterSource = ref<string>('all')
const showFavoritesOnly = ref(false)

onMounted(() => {
  loadPlugins()
})

const filteredPlugins = computed(() => {
  return plugins.value.filter(plugin => {
    const matchesSearch = searchQuery.value === '' ||
      plugin.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      plugin.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      plugin.author.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesCompatibility = filterCompatibility.value === 'all' ||
      plugin.compatibility === filterCompatibility.value

    const matchesSource = filterSource.value === 'all' ||
      plugin.source.source_type === filterSource.value

    const matchesFavorite = !showFavoritesOnly.value || plugin.is_favorite === true

    return matchesSearch && matchesCompatibility && matchesSource && matchesFavorite
  })
})

const favoritePlugins = computed(() => {
  return plugins.value.filter(p => p.is_favorite).length
})

const showPluginDescription = (plugin: Plugin) => {
  selectedPlugin.value = plugin
  showPluginDetail.value = true
}

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

const removePlugin = async (pluginId: string) => {
  try {
    await api.removePlugin(pluginId)
    toast.success('插件已删除')
    await loadPlugins()
  } catch (error) {
    toast.error(`删除插件失败: ${error}`)
  }
}

const toggleFavorite = async (plugin: Plugin) => {
  try {
    const newState = await api.togglePluginFavorite(plugin.plugin_id)
    plugin.is_favorite = newState
    toast.success(newState ? '已添加到收藏' : '已取消收藏')
  } catch (error) {
    toast.error(`操作失败: ${error}`)
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
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">插件仓库</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="importFromProjects"
          :disabled="isLoading"
          class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors disabled:opacity-50 text-sm"
        >
          从项目导入
        </button>
        <button
          @click="importFromLocal"
          :disabled="isLoading"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 text-sm"
        >
          从目录导入
        </button>
        <button
          @click="importFromFile"
          :disabled="isLoading"
          class="px-4 py-2 bg-teal-600 text-white rounded-lg hover:bg-teal-700 transition-colors disabled:opacity-50 text-sm"
        >
          从文件导入
        </button>
        <button
          @click="showGitDialog = true"
          :disabled="isLoading"
          class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 text-sm"
        >
          从 Git 导入
        </button>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
      <div class="flex flex-col lg:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索插件名称、描述或作者..."
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          />
        </div>
        <div class="flex flex-wrap gap-2 items-center">
          <select
            v-model="filterCompatibility"
            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          >
            <option value="all">全部版本</option>
            <option value="Godot4">Godot 4</option>
            <option value="Godot3">Godot 3</option>
            <option value="Both">通用</option>
          </select>
          <select
            v-model="filterSource"
            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          >
            <option value="all">全部来源</option>
            <option value="Local">本地</option>
            <option value="Git">Git</option>
            <option value="AssetLibrary">AssetLibrary</option>
          </select>
          <button
            @click="showFavoritesOnly = !showFavoritesOnly"
            :class="[
              'px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              showFavoritesOnly
                ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400'
                : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
            ]"
          >
            <span class="flex items-center gap-1">
              <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
              </svg>
              {{ favoritePlugins }} 收藏
            </span>
          </button>
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="filteredPlugins.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">暂无插件</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        从本地目录、文件或 Git 仓库导入插件
      </p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <div
        v-for="plugin in filteredPlugins"
        :key="plugin.plugin_id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-lg transition-shadow p-4"
      >
        <div class="flex items-start justify-between min-w-0">
          <div class="min-w-0 flex-1 cursor-pointer" @click="showPluginDescription(plugin)">
            <div class="flex items-center gap-2">
              <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 truncate">
                {{ plugin.name }}
              </h3>
              <button
                @click.stop="toggleFavorite(plugin)"
                :class="[
                  'p-1 rounded transition-colors',
                  plugin.is_favorite
                    ? 'text-yellow-500 hover:text-yellow-600'
                    : 'text-gray-400 hover:text-yellow-500'
                ]"
              >
                <svg class="w-5 h-5" :fill="plugin.is_favorite ? 'currentColor' : 'none'" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                </svg>
              </button>
            </div>
            <p 
              class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2"
              :title="plugin.description || '无描述'"
            >
              {{ plugin.description || '无描述' }}
            </p>
          </div>
          <button
            @click.stop="removePlugin(plugin.plugin_id)"
            class="text-red-600 hover:text-red-800 ml-2"
          >
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="mt-3 flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
          <span>v{{ plugin.versions[0]?.version || '1.0.0' }}</span>
          <span>{{ plugin.author || '未知作者' }}</span>
        </div>
        <div class="mt-2 flex items-center gap-2 flex-wrap">
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
            {{ plugin.source.source_type === 'Local' ? '本地' : plugin.source.source_type === 'Git' ? 'Git' : 'AssetLibrary' }}
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

    <div v-if="showPluginDetail && selectedPlugin" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-lg shadow-xl">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
          {{ selectedPlugin.name }}
        </h3>
        <div class="mb-4">
          <span class="text-sm text-gray-500 dark:text-gray-400">
            作者: {{ selectedPlugin.author || '未知作者' }}
          </span>
          <span class="mx-2 text-gray-300">|</span>
          <span class="text-sm text-gray-500 dark:text-gray-400">
            版本: v{{ selectedPlugin.versions[0]?.version || '1.0.0' }}
          </span>
        </div>
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">描述</h4>
          <p class="text-sm text-gray-600 dark:text-gray-400 whitespace-pre-wrap bg-gray-50 dark:bg-gray-700 rounded-lg p-3 max-h-60 overflow-y-auto">
            {{ selectedPlugin.description || '无描述' }}
          </p>
        </div>
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">来源</h4>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            {{ selectedPlugin.source.source_type === 'Local' ? '本地目录' : selectedPlugin.source.source_type === 'Git' ? 'Git 仓库' : 'AssetLibrary' }}
            <span v-if="selectedPlugin.source.url" class="block text-xs mt-1 break-all">{{ selectedPlugin.source.url }}</span>
          </p>
        </div>
        <div class="flex justify-end">
          <button
            @click="showPluginDetail = false; selectedPlugin = null"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            关闭
          </button>
        </div>
      </div>
    </div>
  </div>
</template>