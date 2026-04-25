<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { api } from '@/api'
import type { Settings, LogEntry, TeamSharedConfig, Project } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/composables/useI18n'
import { useTheme } from '@/composables/useTheme'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { useOnboarding } from '@/composables/useOnboarding'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const { t, setLocale } = useI18n()
const { setTheme, initTheme } = useTheme()
const settings = ref<Settings>({ scan_directories: [], mount_strategy: 'Symlink', language: 'zh-CN', theme: 'system', auto_scan_on_startup: true, auto_discover_engines: true })
const isLoading = ref(false)
const logs = ref<LogEntry[]>([])
const showLogs = ref(false)
const showBackupDialog = ref(false)
const backupPath = ref('')
const isBackingUp = ref(false)
const isRestoring = ref(false)
const showTeamConfigDialog = ref(false)
const teamConfigs = ref<TeamSharedConfig[]>([])
const projects = ref<Project[]>([])
const showExportDialog = ref(false)
const exportConfigName = ref('')
const exportConfigDescription = ref('')
const selectedProjectIds = ref<string[]>([])
const isExporting = ref(false)
const isImporting = ref(false)

onMounted(() => { initTheme(); loadSettings(); loadTeamConfigs(); loadProjects() })

const loadSettings = async () => {
  isLoading.value = true
  try {
    const result = await api.getSettings()
    settings.value = { scan_directories: result.scan_directories || [], mount_strategy: result.mount_strategy || 'Symlink', language: result.language || 'zh-CN', theme: result.theme || 'system', auto_scan_on_startup: result.auto_scan_on_startup ?? true, auto_discover_engines: result.auto_discover_engines ?? true }
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

const selectBackupPath = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: '选择备份目录' })
    if (selected && typeof selected === 'string') {
      backupPath.value = selected
    }
  } catch (error) { toast.error(`选择目录失败: ${error}`) }
}

const performBackup = async () => {
  if (!backupPath.value) {
    toast.warning('请先选择备份目录')
    return
  }
  isBackingUp.value = true
  try {
    const result = await api.backupData(backupPath.value)
    toast.success(result)
    showBackupDialog.value = false
  } catch (error) {
    toast.error(`备份失败: ${error}`)
  } finally {
    isBackingUp.value = false
  }
}

const performRestore = async () => {
  if (!backupPath.value) {
    toast.warning('请先选择备份目录')
    return
  }
  isRestoring.value = true
  try {
    const result = await api.restoreData(backupPath.value)
    toast.success(result)
    await loadSettings()
    showBackupDialog.value = false
  } catch (error) {
    toast.error(`恢复失败: ${error}`)
  } finally {
    isRestoring.value = false
  }
}

const loadTeamConfigs = async () => {
  try {
    teamConfigs.value = await api.getTeamConfigs()
  } catch (error) {
    console.error('Failed to load team configs:', error)
  }
}

const loadProjects = async () => {
  try {
    projects.value = await api.getProjects()
  } catch (error) {
    console.error('Failed to load projects:', error)
  }
}

const openExportDialog = () => {
  exportConfigName.value = ''
  exportConfigDescription.value = ''
  selectedProjectIds.value = []
  showExportDialog.value = true
}

const exportTeamConfig = async () => {
  if (!exportConfigName.value) {
    toast.warning('请输入配置名称')
    return
  }
  if (selectedProjectIds.value.length === 0) {
    toast.warning('请选择至少一个项目')
    return
  }
  isExporting.value = true
  try {
    await api.exportTeamConfig(exportConfigName.value, exportConfigDescription.value, selectedProjectIds.value)
    toast.success('团队配置导出成功')
    showExportDialog.value = false
    await loadTeamConfigs()
  } catch (error) {
    toast.error(`导出失败: ${error}`)
  } finally {
    isExporting.value = false
  }
}

const importTeamConfig = async (configId: string) => {
  if (selectedProjectIds.value.length === 0) {
    toast.warning('请选择至少一个目标项目')
    return
  }
  isImporting.value = true
  try {
    await api.importTeamConfig(configId, selectedProjectIds.value)
    toast.success('团队配置导入成功')
    showTeamConfigDialog.value = false
  } catch (error) {
    toast.error(`导入失败: ${error}`)
  } finally {
    isImporting.value = false
  }
}

const showDeleteTeamConfigConfirm = ref(false)
const deleteTeamConfigId = ref('')

useDialogEscape(showLogs)
useDialogEscape(showBackupDialog)
useDialogEscape(showTeamConfigDialog)
useDialogEscape(showExportDialog)

const confirmDeleteTeamConfig = (configId: string) => {
  deleteTeamConfigId.value = configId
  showDeleteTeamConfigConfirm.value = true
}

const onDeleteTeamConfigConfirm = async () => {
  try {
    await api.deleteTeamConfig(deleteTeamConfigId.value)
    toast.success('团队配置已删除')
    await loadTeamConfigs()
  } catch (error) {
    toast.error(`删除失败: ${error}`)
  }
}

const formatDate = (dateStr: string) => {
  try {
    return new Date(dateStr).toLocaleString('zh-CN')
  } catch {
    return dateStr
  }
}

const { showOnboarding } = useOnboarding()

const resetOnboarding = async () => {
  try {
    const currentSettings = await api.getSettings()
    currentSettings.onboarding_completed = false
    await api.saveSettings(currentSettings)
    showOnboarding()
  } catch (error) {
    toast.error(`重置引导失败: ${error}`)
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('settings.title') }}</h1>
      <div class="flex gap-2">
        <button @click="loadLogs" class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-sm">查看日志</button>
        <button @click="showBackupDialog = true" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm">数据备份与恢复</button>
        <button @click="showTeamConfigDialog = true" class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-sm">团队配置</button>
      </div>
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
        <div class="mt-4 space-y-3">
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="settings.auto_scan_on_startup" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.autoScanOnStartup') }}</span>
          </label>
          <label class="flex items-center gap-3 cursor-pointer">
            <input type="checkbox" v-model="settings.auto_discover_engines" class="w-4 h-4 text-primary-600 rounded" />
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.autoDiscoverEngines') }}</span>
          </label>
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
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('settings.other') }}</h2>
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.showOnboarding') }}</p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{{ t('settings.showOnboardingDesc') }}</p>
          </div>
          <button
            @click="resetOnboarding"
            class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors text-sm"
          >
            {{ t('settings.showOnboarding') }}
          </button>
        </div>
      </div>
      <div class="flex justify-end">
        <button @click="saveSettings" :disabled="isLoading" class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50">{{ t('settings.save') }}</button>
      </div>
    </div>

    <div v-if="showLogs" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showLogs = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-3xl shadow-xl max-h-[80vh] flex flex-col" @click.stop>
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

    <div v-if="showBackupDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showBackupDialog = false; backupPath = ''">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">数据备份与恢复</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          选择备份目录，将复制所有数据到该目录。恢复时会从该目录读取数据覆盖现有数据。
        </p>
        <div class="flex gap-2 mb-4">
          <input
            v-model="backupPath"
            type="text"
            readonly
            placeholder="请选择备份目录"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          />
          <button
            @click="selectBackupPath"
            class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
          >
            浏览
          </button>
        </div>
        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
          <p class="text-xs text-yellow-800 dark:text-yellow-200">
            <strong>注意：</strong>恢复功能会覆盖现有数据，请在恢复前确认备份文件的正确性。
          </p>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            @click="showBackupDialog = false; backupPath = ''"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="performBackup"
            :disabled="isBackingUp || !backupPath"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 transition-colors"
          >
            {{ isBackingUp ? '备份中...' : '备份数据' }}
          </button>
          <button
            @click="performRestore"
            :disabled="isRestoring || !backupPath"
            class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
          >
            {{ isRestoring ? '恢复中...' : '恢复数据' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showTeamConfigDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showTeamConfigDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-2xl shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">团队配置管理</h3>
          <button @click="showTeamConfigDialog = false" class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="mb-4">
          <button
            @click="openExportDialog"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm"
          >
            导出新配置
          </button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div v-if="teamConfigs.length === 0" class="text-center py-8 text-gray-500 dark:text-gray-400">
            暂无团队配置
          </div>
          <div v-else class="space-y-4">
            <div v-for="config in teamConfigs" :key="config.config_id" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <div class="flex justify-between items-start">
                <div>
                  <h4 class="font-medium text-gray-900 dark:text-gray-100">{{ config.name }}</h4>
                  <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{{ config.description || '无描述' }}</p>
                  <p class="text-xs text-gray-400 dark:text-gray-500 mt-2">创建时间: {{ formatDate(config.created_at) }}</p>
                </div>
                <div class="flex gap-2">
                  <button
                    @click="importTeamConfig(config.config_id)"
                    :disabled="isImporting || projects.length === 0"
                    class="px-3 py-1 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 transition-colors text-sm"
                  >
                    导入
                  </button>
                  <button
                    @click="confirmDeleteTeamConfig(config.config_id)"
                    class="px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700 text-sm"
                  >
                    删除
                  </button>
                </div>
              </div>
              <div class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                包含 {{ config.bindings.length }} 个插件绑定, {{ config.engine_bindings.length }} 个引擎绑定
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-end mt-4">
          <button
            @click="showTeamConfigDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

    <div v-if="showExportDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showExportDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">导出团队配置</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">配置名称</label>
            <input
              v-model="exportConfigName"
              type="text"
              placeholder="例如: 项目A标准配置"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">描述（可选）</label>
            <input
              v-model="exportConfigDescription"
              type="text"
              placeholder="配置描述"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">选择项目</label>
            <div class="space-y-2 max-h-40 overflow-y-auto bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
              <div v-for="project in projects" :key="project.project_id" class="flex items-center gap-2">
                <input
                  type="checkbox"
                  :value="project.project_id"
                  v-model="selectedProjectIds"
                  class="w-4 h-4 text-primary-600 rounded"
                />
                <span class="text-sm text-gray-900 dark:text-gray-100">{{ project.name }}</span>
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showExportDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="exportTeamConfig"
            :disabled="isExporting || !exportConfigName || selectedProjectIds.length === 0"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ isExporting ? '导出中...' : '导出' }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteTeamConfigConfirm"
      title="确认删除团队配置"
      description="确定要删除此团队配置吗？此操作不可撤销。"
      confirm-text="确认删除"
      @confirm="onDeleteTeamConfigConfirm"
    />
  </div>
</template>