<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useFileManager } from '@/composables/useFileManager'
import type { ExportTemplateInfo, BuiltinExportPreset, BuildRecord, ExportPlatform, Project } from '@/types'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const mcpExePath = ref('harbor-mcp-server')

async function resolveMcpExePath() {
  try {
    const paths = await api.getStoragePaths()
    if (paths.app_data_dir) {
      const appDir = paths.app_data_dir.replace(/[/\\]GodotHarbor[/\\]?$/, '')
      const sep = navigator.platform.startsWith('Win') ? '\\' : '/'
      mcpExePath.value = `${appDir}${sep}harbor-mcp-server${navigator.platform.startsWith('Win') ? '.exe' : ''}`
    }
  } catch { /* fallback to default */ }
}

const toast = useToast()
const { t } = useI18n()
const { openInFileManager } = useFileManager()

const activeTab = ref<'templates' | 'presets' | 'build' | 'ci' | 'mcp'>('templates')
const exportTemplates = ref<ExportTemplateInfo[]>([])
const builtinPresets = ref<BuiltinExportPreset[]>([])
const buildRecords = ref<BuildRecord[]>([])
const projects = ref<Project[]>([])
const isLoading = ref(false)
const downloadingVersion = ref<string | null>(null)
const building = ref(false)
const selectedProjectId = ref('')
const selectedPlatform = ref<ExportPlatform>('Windows')
const deleteTarget = ref<{ version: string; mono: boolean } | null>(null)

const ciProvider = ref<'github-actions' | 'gitlab-ci'>('github-actions')
const ciPlatforms = ref<string[]>(['windows', 'web'])
const ciGodotVersion = ref('')
const generatedConfig = ref('')
const presetProjectId = ref('')

const platformOptions: { value: ExportPlatform; label: string }[] = [
  { value: 'Windows', label: 'Windows' },
  { value: 'Web', label: 'Web' },
  { value: 'Linux', label: 'Linux' },
  { value: 'MacOS', label: 'macOS' },
  { value: 'Android', label: 'Android' },
  { value: 'IOS', label: 'iOS' },
]

const ciPlatformOptions = [
  { value: 'windows', label: 'Windows' },
  { value: 'web', label: 'Web' },
  { value: 'linux', label: 'Linux' },
  { value: 'macos', label: 'macOS' },
]

const mcpServerRunning = ref(false)
const buildProgress = ref<{ stage: string; progress: number; message: string } | null>(null)
const downloadProgress = ref<{ version: string; stage: string; progress: number; message: string } | null>(null)
const importPresetJson = ref('')
const mcpClaudeConfig = computed(() => {
  return JSON.stringify({
    mcpServers: {
      'godot-harbor': {
        command: mcpExePath.value,
        args: [],
        transport: 'stdio'
      }
    }
  }, null, 2)
})
const mcpCursorConfig = computed(() => {
  return JSON.stringify({
    mcpServers: {
      'godot-harbor': {
        command: mcpExePath.value,
        args: [],
        transport: 'stdio'
      }
    }
  }, null, 2)
})

let unlistenProgress: UnlistenFn | null = null
let unlistenDownloadProgress: UnlistenFn | null = null

async function loadData() {
  isLoading.value = true
  try {
    const [templates, presets, records, projs] = await Promise.all([
      api.listExportTemplates(),
      api.getBuiltinExportPresets(),
      api.getBuildRecords(),
      api.getProjects(),
    ])
    exportTemplates.value = templates
    builtinPresets.value = presets
    buildRecords.value = records
    projects.value = projs
    if (projs.length > 0 && !selectedProjectId.value) {
      selectedProjectId.value = projs[0].project_id
      presetProjectId.value = projs[0].project_id
    }
    if (projs.length > 0 && !ciGodotVersion.value) {
      const proj = projs[0]
      ciGodotVersion.value = proj.godot_version || '4.4.1'
    }
  } catch (e) {
    toast.error(String(e))
  } finally {
    isLoading.value = false
  }
}

async function downloadTemplate(version: string, mono: boolean) {
  downloadingVersion.value = version
  try {
    await api.downloadExportTemplate(version, mono)
    toast.success(t('build.templateDownloaded'))
    await loadData()
  } catch (e) {
    toast.error(String(e))
  } finally {
    downloadingVersion.value = null
  }
}

async function deleteTemplate() {
  if (!deleteTarget.value) return
  try {
    await api.deleteExportTemplate(deleteTarget.value.version, deleteTarget.value.mono)
    toast.success(t('build.templateDeleted'))
    deleteTarget.value = null
    await loadData()
  } catch (e) {
    toast.error(String(e))
  }
}

async function applyPreset(preset: BuiltinExportPreset) {
  if (!presetProjectId.value) {
    toast.error(t('build.selectProject'))
    return
  }
  try {
    await api.saveExportPresetToHarbor(presetProjectId.value, preset.platform, preset.name, preset.config)
    toast.success(t('build.presetSaved'))
  } catch (e) {
    toast.error(String(e))
  }
}

async function exportPreset(preset: BuiltinExportPreset) {
  try {
    const json = await api.exportPresetToJson(preset as unknown as Record<string, unknown>)
    await navigator.clipboard.writeText(json)
    toast.success(t('build.presetExported') || '预设已复制到剪贴板')
  } catch (e) {
    toast.error(String(e))
  }
}

async function importPreset() {
  if (!presetProjectId.value || !importPresetJson.value.trim()) return
  try {
    await api.importPresetFromJson(presetProjectId.value, importPresetJson.value.trim())
    toast.success(t('build.presetImported') || '预设导入成功')
    importPresetJson.value = ''
  } catch (e) {
    toast.error(String(e))
  }
}

async function startBuild() {
  if (!selectedProjectId.value) {
    toast.error(t('build.selectProject'))
    return
  }
  building.value = true
  try {
    const record = await api.buildProject(selectedProjectId.value, selectedPlatform.value)
    if (record.status === 'Success') {
      toast.success(t('build.buildSuccess'))
    } else {
      toast.error(t('build.buildFailed'))
    }
    await loadData()
  } catch (e) {
    toast.error(String(e))
  } finally {
    building.value = false
  }
}

async function retryBuild(record: any) {
  selectedProjectId.value = record.project_id
  const platformMap: Record<string, string> = { 'Windows': 'windows', 'macOS': 'macos', 'Linux': 'linux', 'Web': 'web', 'Android': 'android', 'iOS': 'ios' }
  selectedPlatform.value = platformMap[record.platform] || record.platform
  await startBuild()
}

async function generateCi() {
  if (!selectedProjectId.value) {
    toast.error(t('build.selectProject'))
    return
  }
  try {
    let config: string
    if (ciProvider.value === 'github-actions') {
      config = await api.generateGithubActions(selectedProjectId.value, ciPlatforms.value, ciGodotVersion.value)
    } else {
      config = await api.generateGitlabCi(selectedProjectId.value, ciPlatforms.value, ciGodotVersion.value)
    }
    generatedConfig.value = config
    toast.success(t('build.configGenerated'))
  } catch (e) {
    toast.error(String(e))
  }
}

async function writeCiConfig() {
  if (!selectedProjectId.value || !generatedConfig.value) return
  try {
    await api.writeCiConfig(selectedProjectId.value, ciProvider.value, generatedConfig.value)
    toast.success(t('build.configWritten'))
  } catch (e) {
    toast.error(String(e))
  }
}

async function removeBuildRecord(buildId: string) {
  try {
    await api.deleteBuildRecord(buildId)
    buildRecords.value = buildRecords.value.filter(r => r.build_id !== buildId)
  } catch (e) {
    toast.error(String(e))
  }
}

function statusClass(status: string) {
  switch (status) {
    case 'Success': return 'text-green-500'
    case 'Failed': return 'text-red-500'
    case 'Running': return 'text-blue-500'
    case 'Pending': return 'text-yellow-500'
    default: return 'text-gray-500'
  }
}

function statusText(status: string) {
  switch (status) {
    case 'Success': return t('build.statusSuccess')
    case 'Failed': return t('build.statusFailed')
    case 'Running': return t('build.statusRunning')
    case 'Pending': return t('build.statusPending')
    case 'Cancelled': return t('build.statusCancelled')
    default: return status
  }
}

function formatSize(bytes: number | null) {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleString()
  } catch {
    return dateStr
  }
}

async function startMcpServer() {
  try {
    await api.startMcpServer()
    mcpServerRunning.value = true
    toast.success(t('mcp.serverRunning'))
  } catch (e) {
    toast.error(String(e))
  }
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.success(t('mcp.configCopied'))
  } catch {
    toast.error('Failed to copy')
  }
}

onMounted(async () => {
  await loadData()
  resolveMcpExePath()
  unlistenProgress = await listen('build-progress', (event: any) => {
    buildProgress.value = event.payload
    if (event.payload.stage === 'complete' || event.payload.stage === 'failed') {
      setTimeout(() => { buildProgress.value = null }, 3000)
    }
  })
  unlistenDownloadProgress = await listen('export-template-download-progress', (event: any) => {
    downloadProgress.value = event.payload
    if (event.payload.stage === 'complete') {
      setTimeout(() => { downloadProgress.value = null; loadData() }, 1500)
    }
  })
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenDownloadProgress?.()
})
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="px-6 pt-6 pb-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{{ t('build.title') }}</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{{ t('build.subtitle') }}</p>
    </div>

    <div class="px-6 flex gap-1 border-b border-gray-200 dark:border-gray-700 mb-4">
      <button
        v-for="tab in ([
          { key: 'templates', label: t('build.exportTemplates') },
          { key: 'presets', label: t('build.presets') },
          { key: 'build', label: t('build.buildProject') },
          { key: 'ci', label: t('build.ciConfig') },
          { key: 'mcp', label: t('build.mcpServer') || 'MCP Server' },
        ] as const)"
        :key="tab.key"
        class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
        :class="activeTab === tab.key
          ? 'border-blue-500 text-blue-600 dark:text-blue-400'
          : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-6 pb-6">
      <!-- Export Templates Tab -->
      <div v-if="activeTab === 'templates'">
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">{{ t('build.exportTemplatesDesc') }}</p>
        <div v-if="exportTemplates.length === 0" class="text-center py-12 text-gray-400">
          {{ t('build.notInstalled') }}
        </div>
        <div v-else class="grid gap-3">
          <div
            v-for="tmpl in exportTemplates"
            :key="tmpl.version + (tmpl.mono ? '-mono' : '')"
            class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700"
          >
            <div class="flex items-center gap-3">
              <span
                class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
                :class="tmpl.installed
                  ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                  : 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400'"
              >
                {{ tmpl.installed ? t('build.installed') : t('build.notInstalled') }}
              </span>
              <span class="font-medium text-gray-900 dark:text-white">{{ tmpl.version }}</span>
              <span v-if="tmpl.mono" class="text-xs text-purple-600 dark:text-purple-400 bg-purple-50 dark:bg-purple-900/20 px-1.5 py-0.5 rounded">
                {{ t('build.mono') }}
              </span>
              <span v-if="tmpl.file_size" class="text-xs text-gray-400">{{ formatSize(tmpl.file_size) }}</span>
            </div>
            <div class="flex items-center gap-2">
              <button
                v-if="!tmpl.installed"
                class="px-3 py-1.5 text-sm font-medium text-white bg-blue-500 hover:bg-blue-600 rounded-lg transition-colors disabled:opacity-50"
                :disabled="downloadingVersion === tmpl.version"
                @click="downloadTemplate(tmpl.version, tmpl.mono)"
              >
                {{ downloadingVersion === tmpl.version ? t('build.downloading') : t('build.download') }}
              </button>
              <button
                v-if="tmpl.installed"
                class="px-3 py-1.5 text-sm font-medium text-red-600 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                @click="deleteTarget = { version: tmpl.version, mono: tmpl.mono }"
              >
                {{ t('build.delete') }}
              </button>
            </div>
          </div>
        </div>

        <div v-if="downloadProgress" class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-xl p-4 mt-4">
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-5 w-5 border-2 border-blue-600 border-t-transparent flex-shrink-0" v-if="downloadProgress.stage !== 'complete'"></div>
            <svg v-else class="w-5 h-5 text-green-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium text-blue-800 dark:text-blue-300">{{ downloadProgress.message }}</p>
              <div v-if="downloadProgress.stage !== 'complete'" class="mt-2 w-full bg-blue-200 dark:bg-blue-800 rounded-full h-1.5">
                <div class="bg-blue-600 h-1.5 rounded-full transition-all" :style="{ width: (downloadProgress.progress * 100) + '%' }"></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Export Presets Tab -->
      <div v-if="activeTab === 'presets'">
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">{{ t('build.presetsDesc') }}</p>
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('build.selectProject') }}</label>
          <select
            v-model="presetProjectId"
            class="w-full max-w-md px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
          >
            <option v-for="p in projects" :key="p.project_id" :value="p.project_id">{{ p.name }}</option>
          </select>
        </div>
        <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">{{ t('build.builtinPresets') }}</h3>
        <div class="grid gap-3 sm:grid-cols-2">
          <div
            v-for="preset in builtinPresets"
            :key="preset.platform + preset.name"
            class="p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700"
          >
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-gray-900 dark:text-white">{{ preset.name }}</span>
              <span class="text-xs text-gray-500 dark:text-gray-400">{{ preset.platform }}</span>
            </div>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">{{ preset.description }}</p>
            <div class="flex items-center gap-2">
              <button
                class="px-3 py-1.5 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30 rounded-lg transition-colors"
                @click="applyPreset(preset)"
              >
                {{ t('build.applyPreset') }}
              </button>
              <button
                class="px-3 py-1.5 text-sm font-medium text-gray-600 hover:text-gray-700 bg-gray-50 hover:bg-gray-100 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600 rounded-lg transition-colors"
                @click="exportPreset(preset)"
              >
                {{ t('build.exportPreset') || '导出' }}
              </button>
            </div>
          </div>
        </div>

        <div class="mt-6 p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
          <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">{{ t('build.importPresetTitle') || '导入预设' }}</h3>
          <div class="flex gap-2">
            <input
              v-model="importPresetJson"
              type="text"
              :placeholder="t('build.importPresetPlaceholder') || '粘贴预设JSON'"
              class="flex-1 px-3 py-2 text-sm bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg"
            />
            <button
              class="px-4 py-2 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30 rounded-lg transition-colors disabled:opacity-50"
              :disabled="!importPresetJson.trim() || !presetProjectId"
              @click="importPreset"
            >
              {{ t('build.importPreset') || '导入' }}
            </button>
          </div>
        </div>
      </div>

      <!-- Build Tab -->
      <div v-if="activeTab === 'build'">
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">{{ t('build.buildDesc') }}</p>
        <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-6">
          <div class="grid gap-4 sm:grid-cols-3">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('build.selectProject') }}</label>
              <select
                v-model="selectedProjectId"
                class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
              >
                <option v-for="p in projects" :key="p.project_id" :value="p.project_id">{{ p.name }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('build.selectPlatform') }}</label>
              <select
                v-model="selectedPlatform"
                class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
              >
                <option v-for="opt in platformOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
              </select>
            </div>
            <div class="flex items-end">
              <button
                class="w-full px-4 py-2 text-sm font-medium text-white bg-blue-500 hover:bg-blue-600 rounded-lg transition-colors disabled:opacity-50"
                :disabled="building || !selectedProjectId"
                @click="startBuild"
              >
                {{ building ? t('build.building') : t('build.startBuild') }}
              </button>
            </div>
          </div>
        </div>

        <div v-if="buildProgress" class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-xl p-4 mb-6">
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-5 w-5 border-2 border-blue-600 border-t-transparent flex-shrink-0" v-if="buildProgress.stage !== 'complete' && buildProgress.stage !== 'failed'"></div>
            <svg v-else-if="buildProgress.stage === 'complete'" class="w-5 h-5 text-green-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            <svg v-else class="w-5 h-5 text-red-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium text-blue-800 dark:text-blue-300">{{ buildProgress.message }}</p>
              <div v-if="buildProgress.stage !== 'complete' && buildProgress.stage !== 'failed'" class="mt-2 w-full bg-blue-200 dark:bg-blue-800 rounded-full h-1.5">
                <div class="bg-blue-600 h-1.5 rounded-full transition-all" :style="{ width: (buildProgress.progress * 100) + '%' }"></div>
              </div>
            </div>
          </div>
        </div>

        <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-3">{{ t('build.buildHistory') }}</h3>
        <div v-if="buildRecords.length === 0" class="text-center py-12 text-gray-400">
          {{ t('build.noHistory') }}
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="record in buildRecords.slice().reverse()"
            :key="record.build_id"
            class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700"
          >
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="font-medium text-gray-900 dark:text-white">{{ record.project_name }}</span>
                <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400">{{ record.platform }}</span>
                <span class="text-xs font-medium" :class="statusClass(record.status)">{{ statusText(record.status) }}</span>
              </div>
              <div class="text-xs text-gray-500 dark:text-gray-400 flex gap-3">
                <span>{{ record.engine_version }}</span>
                <span>{{ formatDate(record.started_at) }}</span>
                <span v-if="record.duration_secs">{{ t('build.duration') }}: {{ record.duration_secs }}{{ t('build.seconds') }}</span>
              </div>
              <p v-if="record.error_message" class="text-xs text-red-500 mt-1 truncate" :title="record.error_message">
                {{ record.error_message.substring(0, 100) }}
              </p>
            </div>
            <div class="flex items-center gap-2 ml-4">
              <button
                v-if="record.status === 'Failed'"
                class="text-xs text-amber-500 hover:text-amber-600"
                @click="retryBuild(record)"
              >
                {{ t('build.retry') || '重试' }}
              </button>
              <button
                v-if="record.status === 'Success' && record.output_path"
                class="text-xs text-blue-500 hover:text-blue-600"
                @click="openInFileManager(record.output_path)"
              >
                {{ t('build.openOutput') }}
              </button>
              <button
                class="text-xs text-red-400 hover:text-red-500"
                @click="removeBuildRecord(record.build_id)"
              >
                {{ t('build.delete') }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- CI/CD Tab -->
      <div v-if="activeTab === 'ci'">
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">{{ t('build.ciDesc') }}</p>
        <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-6">
          <div class="grid gap-4 sm:grid-cols-2 mb-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('build.selectProject') }}</label>
              <select
                v-model="selectedProjectId"
                class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
              >
                <option v-for="p in projects" :key="p.project_id" :value="p.project_id">{{ p.name }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('build.provider') }}</label>
              <select
                v-model="ciProvider"
                class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
              >
                <option value="github-actions">{{ t('build.githubActions') }}</option>
                <option value="gitlab-ci">{{ t('build.gitlabCi') }}</option>
              </select>
            </div>
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('build.godotVersion') }}</label>
            <input
              v-model="ciGodotVersion"
              type="text"
              class="w-full max-w-xs px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
              placeholder="4.4.1"
            />
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('build.selectPlatforms') }}</label>
            <div class="flex flex-wrap gap-2">
              <label
                v-for="opt in ciPlatformOptions"
                :key="opt.value"
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-sm cursor-pointer transition-colors"
                :class="ciPlatforms.includes(opt.value)
                  ? 'border-blue-500 bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-400'
                  : 'border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400'"
              >
                <input type="checkbox" :value="opt.value" v-model="ciPlatforms" class="sr-only" />
                {{ opt.label }}
              </label>
            </div>
          </div>
          <div class="flex gap-2">
            <button
              class="px-4 py-2 text-sm font-medium text-white bg-blue-500 hover:bg-blue-600 rounded-lg transition-colors disabled:opacity-50"
              :disabled="!selectedProjectId || ciPlatforms.length === 0"
              @click="generateCi"
            >
              {{ t('build.generate') }}
            </button>
          </div>
        </div>

        <div v-if="generatedConfig" class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
              {{ ciProvider === 'github-actions' ? '.github/workflows/build.yml' : '.gitlab-ci.yml' }}
            </h3>
            <button
              class="px-3 py-1.5 text-sm font-medium text-green-600 hover:text-green-700 bg-green-50 hover:bg-green-100 dark:bg-green-900/20 dark:text-green-400 dark:hover:bg-green-900/30 rounded-lg transition-colors"
              @click="writeCiConfig"
            >
              {{ t('build.writeConfig') }}
            </button>
          </div>
          <pre class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 text-xs text-gray-800 dark:text-gray-200 overflow-x-auto max-h-96 overflow-y-auto">{{ generatedConfig }}</pre>
        </div>
      </div>

      <!-- MCP Server Tab -->
      <div v-if="activeTab === 'mcp'">
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">{{ t('mcp.subtitle') }}</p>

        <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-6">
          <div class="flex items-center justify-between mb-4">
            <div>
              <h3 class="text-sm font-medium text-gray-900 dark:text-white">{{ t('mcp.title') }}</h3>
              <p class="text-xs mt-1" :class="mcpServerRunning ? 'text-green-500' : 'text-gray-400'">
                {{ mcpServerRunning ? t('mcp.serverRunning') : t('mcp.serverStopped') }}
              </p>
            </div>
            <button
              class="px-4 py-2 text-sm font-medium text-white bg-blue-500 hover:bg-blue-600 rounded-lg transition-colors disabled:opacity-50"
              :disabled="mcpServerRunning"
              @click="startMcpServer"
            >
              {{ t('mcp.startServer') }}
            </button>
          </div>

          <div class="grid gap-4 sm:grid-cols-3">
            <div class="p-3 bg-gray-50 dark:bg-gray-900 rounded-lg">
              <h4 class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('mcp.resources') }} (7)</h4>
              <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('mcp.resourcesList') }}</p>
            </div>
            <div class="p-3 bg-gray-50 dark:bg-gray-900 rounded-lg">
              <h4 class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('mcp.tools') }} (7)</h4>
              <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('mcp.toolsList') }}</p>
            </div>
            <div class="p-3 bg-gray-50 dark:bg-gray-900 rounded-lg">
              <h4 class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t('mcp.prompts') }} (3)</h4>
              <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('mcp.promptsList') }}</p>
            </div>
          </div>
        </div>

        <div class="grid gap-6 sm:grid-cols-2">
          <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
            <div class="flex items-center justify-between mb-3">
              <h3 class="text-sm font-medium text-gray-900 dark:text-white">{{ t('mcp.claudeDesktop') }}</h3>
              <button
                class="px-3 py-1.5 text-xs font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30 rounded-lg transition-colors"
                @click="copyToClipboard(mcpClaudeConfig)"
              >
                {{ t('mcp.copyConfig') }}
              </button>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mb-3">{{ t('mcp.claudeDesktopDesc') }}</p>
            <pre class="bg-gray-50 dark:bg-gray-900 rounded-lg p-3 text-xs text-gray-800 dark:text-gray-200 overflow-x-auto max-h-48 overflow-y-auto">{{ mcpClaudeConfig }}</pre>
          </div>

          <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
            <div class="flex items-center justify-between mb-3">
              <h3 class="text-sm font-medium text-gray-900 dark:text-white">{{ t('mcp.cursor') }}</h3>
              <button
                class="px-3 py-1.5 text-xs font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30 rounded-lg transition-colors"
                @click="copyToClipboard(mcpCursorConfig)"
              >
                {{ t('mcp.copyConfig') }}
              </button>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mb-3">{{ t('mcp.cursorDesc') }}</p>
            <pre class="bg-gray-50 dark:bg-gray-900 rounded-lg p-3 text-xs text-gray-800 dark:text-gray-200 overflow-x-auto max-h-48 overflow-y-auto">{{ mcpCursorConfig }}</pre>
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      :model-value="!!deleteTarget"
      :title="t('build.deleteTemplate')"
      :description="t('build.deleteTemplateConfirm')"
      @confirm="deleteTemplate"
      @update:model-value="(v: boolean) => { if (!v) deleteTarget = null }"
    />
  </div>
</template>
