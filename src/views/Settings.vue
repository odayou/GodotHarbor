<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'
import type { Settings } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'

const settings = ref<Settings>({
  scan_directories: ['D:\\'],
  mount_strategy: 'Symlink',
  language: 'zh-CN',
  theme: 'light'
})

const isLoading = ref(false)
const debugLog = ref<string[]>([])

onMounted(() => {
  loadSettings()
})

const addDebugLog = (message: string) => {
  const timestamp = new Date().toLocaleTimeString()
  debugLog.value.push(`[${timestamp}] ${message}`)
  console.log(message)
}

const loadSettings = async () => {
  isLoading.value = true
  addDebugLog('开始加载设置...')
  try {
    const result = await api.getSettings()
    settings.value = {
      scan_directories: result.scan_directories || [],
      mount_strategy: result.mount_strategy || 'Symlink',
      language: result.language || 'zh-CN',
      theme: result.theme || 'light'
    }
    addDebugLog('设置加载成功')
  } catch (error) {
    addDebugLog(`加载设置失败: ${error}`)
    console.error('加载设置失败:', error)
  } finally {
    isLoading.value = false
  }
}

const addScanDirectory = async () => {
  addDebugLog('开始添加扫描目录...')
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择扫描目录'
    })

    if (selected && typeof selected === 'string') {
      // 确保数组存在
      if (!settings.value.scan_directories) {
        settings.value.scan_directories = []
      }
      if (!settings.value.scan_directories.includes(selected)) {
        settings.value.scan_directories.push(selected)
        addDebugLog(`目录已添加: ${selected}`)
      }
    }
  } catch (error) {
    addDebugLog(`添加目录失败: ${error}`)
    console.error('添加目录失败:', error)
  }
}

const removeScanDirectory = (index: number) => {
  addDebugLog(`删除扫描目录: ${settings.value.scan_directories[index]}`)
  if (settings.value.scan_directories) {
    settings.value.scan_directories.splice(index, 1)
  }
}

const saveSettings = async () => {
  isLoading.value = true
  addDebugLog('开始保存设置...')
  try {
    await api.saveSettings(settings.value)
    addDebugLog('设置保存成功')
  } catch (error) {
    addDebugLog(`保存设置失败: ${error}`)
    console.error('保存设置失败:', error)
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">设置</h1>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">项目扫描</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              扫描目录
            </label>
            <div class="space-y-2">
              <div
                v-for="(dir, index) in settings.scan_directories"
                :key="index"
                class="flex items-center space-x-2"
              >
                <input
                  type="text"
                  readonly
                  :value="dir"
                  class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                />
                <button
                  @click="removeScanDirectory(index)"
                  class="px-3 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                >
                  移除
                </button>
              </div>
              <button
                @click="addScanDirectory"
                class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
              >
                添加目录
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">挂载策略</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              默认挂载方式
            </label>
            <select
              v-model="settings.mount_strategy"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            >
              <option value="Symlink">符号链接（推荐）</option>
              <option value="Junction">目录联接（Windows）</option>
              <option value="Copy">复制文件</option>
            </select>
          </div>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">界面设置</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              语言
            </label>
            <select
              v-model="settings.language"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            >
              <option value="zh-CN">简体中文</option>
              <option value="en-US">English</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              主题
            </label>
            <select
              v-model="settings.theme"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            >
              <option value="light">浅色</option>
              <option value="dark">深色</option>
            </select>
          </div>
        </div>
      </div>

      <div class="flex justify-end">
        <button
          @click="saveSettings"
          :disabled="isLoading"
          class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50"
        >
          保存设置
        </button>
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
  </div>
</template>
