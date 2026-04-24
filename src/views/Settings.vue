<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { api } from '@/api'
import type { Settings } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/composables/useI18n'
import { useTheme } from '@/composables/useTheme'

const toast = useToast()
const { t, setLocale } = useI18n()
const { setTheme, initTheme } = useTheme()
const settings = ref<Settings>({ scan_directories: [], mount_strategy: 'Symlink', language: 'zh-CN', theme: 'system' })
const isLoading = ref(false)

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
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('settings.title') }}</h1>
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
  </div>
</template>
