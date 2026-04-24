<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { api } from '@/api'
import type { Settings, LogEntry } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/composables/useI18n'
import { useTheme } from '@/composables/useTheme'

const toast = useToast()
const { t, setLocale } = useI18n()
const { setTheme, initTheme } = useTheme()
const settings = ref<Settings>({ scan_directories: [], mount_strategy: 'Symlink', language: 'zh-CN', theme: 'system' })
const isLoading = ref(false)
const logs = ref<LogEntry[]>([])
const showLogs = ref(false)

onMounted(() => { initTheme(); loadSettings() })

const loadSettings = async () => {
  isLoading.value = true
  try {
    const result = await api.getSettings()
    settings.value = { scan_directories: result.scan_directories || [], mount_strategy: result.mount_strategy || 'Symlink', language: result.language || 'zh-CN', theme: result.theme || 'system' }
    setLocale(settings.value.language)
    if (['light', 'dark', 'system'].includes(settings.value.theme)) setTheme(settings.value.theme as 'light' | 'dark' | 'system')
  } catch (error) { toast.error(`加载设置失败: ${error}`) }
  finally { isLoading.value = false }
}

watch(() => settings.value.language, (lang) => { setLocale(lang) })
watch(() => settings.value.theme, (theme) => { if (['light', 'dark', 'system'].includes(theme)) setTheme(theme as 'light' | 'dark' | 'system') })

const addScanDirectory = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('settings.scanDirs') })
    if (selected && typeof selected === 'string') {
      if (!settings.value.scan_directories) settings.value.scan_directories = []
      if (!settings.value.scan_directories.includes(selected)) { settings.value.scan_directories.push(selected); toast.info(`已添加目录: ${selected}`) }
      else toast.warning('该目录已存在')
    }
  } catch (error) { toast.error(`添加目录失败: ${error}`) }
}

const removeScanDirectory = (index: number) => { const dir = settings.value.scan_directories[index]; settings.value.scan_directories.splice(index, 1); toast.info(`已移除目录: ${dir}`) }

const saveSettings = async () => {
  isLoading.value = true
  try { await api.saveSettings(settings.value); toast.success('设置保存成功') }
  catch (error) { toast.error(`保存设置失败: ${error}`) }
  finally { isLoading.value = false }
}

const loadLogs = async () => {
  try {
    logs.value = await api.getOperationLogs(50)
    showLogs.value = true
  } catch (error) { toast.error(`加载日志失败: ${error}`) }
}

const copyError = async (log: LogEntry) => {
  try {
    await navigator.clipboard.writeText(log.detail)
    toast.success('已复制到剪贴板')
  } catch { toast.error('复制失败') }
}

const formatTime = (timestamp: string) => {
  try {
    const date = new Date(timestamp)
    return date.toLocaleString('zh-CN')
  } catch { return timestamp }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('settings.title') }}</h1>
      <button @click="loadLogs" class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors text-sm">查看日志</button>
    </div>
    <div v-if="isLoading" class="flex justify-center py-12"><div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div></div>
    <div v-else class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.scan') }}</h2>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.scanDirs') }}</label>
        <div class="space-y-2">
          <div v-for="(dir, index) in settings.scan_directories" :key="index" class="flex items-center space-x-2">
            <input type="text" readonly :value="dir" class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm" />
            <button @click="removeScanDirectory(index)" class="px-3 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm">{{ t('settings.remove') }}</button>
          </div>
          <div v-if="!settings.scan_directories?.length" class="text-sm text-gray-500 dark:text-gray-400 py-2">{{ t('settings.noDirs') }}</div>
          <button @click="addScanDirectory" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm">{{ t('settings.addDir') }}</button>
        </div>
      </div>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.mount') }}</h2>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.mountStrategy') }}</label>
        <select v-model="settings.mount_strategy" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
          <option value="Symlink">{{ t('settings.symlink') }}</option>
          <option value="Junction">{{ t('settings.junction') }}</option>
          <option value="Copy">{{ t('settings.copy') }}</option>
        </select>
      </div>
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.appearance') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.language') }}</label>
            <select v-model="settings.language" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
              <option value="zh-CN">简体中文</option>
              <option value="en">English</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('settings.theme') }}</label>
            <select v-model="settings.theme" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
              <option value="light">{{ t('settings.themeLight') }}</option>
              <option value="dark">{{ t('settings.themeDark') }}</option>
              <option value="system">{{ t('settings.themeSystem') }}</option>
            </select>
          </div>
        </div>
      </div>
      <div class="flex justify-end">
        <button @click="saveSettings" :disabled="isLoading" class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50">{{ t('settings.save') }}</button>
      </div>
    </div>

    <div v-if="showLogs" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-3xl shadow-xl max-h-[80vh] flex flex-col">
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">操作日志</h3>
          <button @click="showLogs = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
          </button>
        </div>
        <div class="flex-1 overflow-y-auto space-y-2">
          <div v-if="logs.length === 0" class="text-center py-8 text-gray-500 dark:text-gray-400">暂无日志记录</div>
          <div v-for="(log, index) in logs" :key="index" :class="['p-3 rounded-lg border', log.level === 'error' ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800' : 'bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-600']">
            <div class="flex justify-between items-start">
              <div class="flex items-center gap-2">
                <span :class="['px-2 py-0.5 rounded text-xs font-medium', log.level === 'error' ? 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300' : 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300']">{{ log.level === 'error' ? '错误' : '成功' }}</span>
                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ log.action }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-500 dark:text-gray-400">{{ formatTime(log.timestamp) }}</span>
                <button v-if="log.level === 'error'" @click="copyError(log)" class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400">复制</button>
              </div>
            </div>
            <p v-if="log.target" class="text-xs text-gray-500 dark:text-gray-400 mt-1">目标: {{ log.target }}</p>
            <p :class="['text-sm mt-1', log.level === 'error' ? 'text-red-700 dark:text-red-300' : 'text-gray-600 dark:text-gray-400']">{{ log.detail }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
