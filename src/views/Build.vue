<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { open } from '@tauri-apps/plugin-dialog'
import { useFileManager } from '@/composables/useFileManager'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { formatSize, formatDate, buildStatusClass, buildStatusText, copyToClipboard } from '@/utils/formatUtils'
import type { ExportTemplateInfo, BuiltinExportPreset, BuildRecord, ExportPlatform, Project } from '@/types'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ProjectSelector from '@/components/ProjectSelector.vue'

const mcpExePath = ref('harbor-mcp-server')
const mcpExeNotFound = ref(false)
const mcpServerRunning = ref(false)
const mcpCapabilities = ref<{ tools: any[]; tools_count: number; resources: any[]; resources_count: number; prompts: any[]; prompts_count: number } | null>(null)
const mcpExpandedSection = ref<'tools' | 'resources' | 'prompts' | null>(null)
const mcpSelectedClient = ref('claude')

async function resolveMcpExePath() {
  try {
    mcpExePath.value = await api.getMcpServerPath()
    mcpExeNotFound.value = false
  } catch {
    mcpExeNotFound.value = true
  }
}

async function loadMcpCapabilities() {
  try {
    mcpCapabilities.value = await api.getMcpCapabilities()
  } catch { /* fallback */ }
}

const toast = useToast()
const { t } = useI18n()
const { openInFileManager } = useFileManager()

const activeTab = ref<'export' | 'build' | 'mcp'>('export')
const exportTemplates = ref<ExportTemplateInfo[]>([])
const builtinPresets = ref<BuiltinExportPreset[]>([])
const buildRecords = ref<BuildRecord[]>([])
const projects = ref<Project[]>([])
const isLoading = ref(false)
const isRefreshing = ref(false)
const downloadingVersion = ref<string | null>(null)
const building = ref(false)
const selectedProjectId = ref('')
const selectedPlatform = ref<ExportPlatform>('Windows')
const deleteTarget = ref<{ version: string; mono: boolean } | null>(null)

const ciProvider = ref<'github-actions' | 'gitlab-ci'>('github-actions')
watch(ciProvider, () => { generatedConfig.value = '' })
const ciPlatforms = ref<string[]>(['windows', 'web'])
const ciGodotVersion = ref('')
const isValidGodotVersion = computed(() => {
  const v = ciGodotVersion.value.trim()
  return /^\d+\.\d+(\.\d+)?/.test(v)
})
const ciProjectId = ref('')
const generatedConfig = ref('')
const presetProjectId = ref('')

watch(ciProjectId, async (newId) => {
  generatedConfig.value = ''
  if (!newId) return
  try {
    const config = await api.readHarborConfig(newId)
    if (config) {
      if (config.ci) {
        if (config.ci.provider) ciProvider.value = config.ci.provider
        if (config.ci.platforms?.length) ciPlatforms.value = config.ci.platforms
      }
      if (config.godot?.version) {
        ciGodotVersion.value = config.godot.version
      } else {
        const proj = projects.value.find(p => p.project_id === newId)
        if (proj?.godot_version) ciGodotVersion.value = proj.godot_version
      }
    } else {
      const proj = projects.value.find(p => p.project_id === newId)
      if (proj?.godot_version) ciGodotVersion.value = proj.godot_version
    }
  } catch { /* ignore */ }
})

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

interface BuildProgressPayload { stage: string; progress: number; message: string }
interface DownloadProgressPayload { version: string; stage: string; progress: number; message: string }

const buildProgress = ref<BuildProgressPayload | null>(null)
const downloadProgress = ref<DownloadProgressPayload | null>(null)
const importPresetJson = ref('')

const mcpConfig = computed(() => {
  return JSON.stringify({
    mcpServers: {
      'godot-harbor': {
        command: mcpExePath.value,
        args: []
      }
    }
  }, null, 2)
})

const mcpClients = computed(() => [
  { key: 'claude', title: t('mcp.claudeDesktop'), desc: t('mcp.claudeDesktopDesc'), configPath: '~/Library/Application Support/Claude/claude_desktop_config.json (macOS)\n%APPDATA%\\Claude\\claude_desktop_config.json (Windows)' },
  { key: 'cursor', title: t('mcp.cursor'), desc: t('mcp.cursorDesc'), configPath: '.cursor/mcp.json' },
  { key: 'vscode', title: 'VS Code (Copilot)', desc: t('mcp.vscodeDesc') || 'Add to VS Code settings.json (mcp.servers section)', configPath: '.vscode/mcp.json or settings.json' },
  { key: 'trae', title: 'Trae', desc: t('mcp.traeDesc') || 'Add to Trae MCP configuration', configPath: '.trae/mcp.json' }
])

let unlistenProgress: UnlistenFn | null = null
let unlistenDownloadProgress: UnlistenFn | null = null
let mcpPollTimer: ReturnType<typeof setInterval> | null = null

async function loadData(force = false) {
  const hasData = exportTemplates.value.length > 0 || buildRecords.value.length > 0
  if (hasData && !force) {
    isRefreshing.value = true
    try {
      const [templates, records, projs] = await Promise.all([
        api.listExportTemplates(),
        api.getBuildRecords(),
        api.getProjects(),
      ])
      exportTemplates.value = templates
      buildRecords.value = records
      projects.value = projs
    } catch (e) {
      toast.error(e)
    } finally {
      isRefreshing.value = false
    }
    return
  }
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
      ciProjectId.value = projs[0].project_id
    }
    if (projs.length > 0 && !ciGodotVersion.value) {
      const proj = projs[0]
      ciGodotVersion.value = proj.godot_version || '4.4.1'
    }
  } catch (e) {
    toast.error(e)
  } finally {
    isLoading.value = false
  }
}

async function downloadTemplate(version: string, mono: boolean) {
  downloadingVersion.value = `${version}-${mono}`
  downloadProgress.value = null
  let stalledTimer: ReturnType<typeof setTimeout> | null = null
  let lastProgressTime = Date.now()

  const unlisten = await listen<DownloadProgressPayload>('export-template-download-progress', (event) => {
    downloadProgress.value = event.payload
    lastProgressTime = Date.now()
    if (event.payload.stage === 'complete') {
      if (stalledTimer) clearTimeout(stalledTimer)
      setTimeout(() => { downloadProgress.value = null; loadData() }, 1500)
    } else if (event.payload.stage === 'failed') {
      if (stalledTimer) clearTimeout(stalledTimer)
      setTimeout(() => { downloadProgress.value = null; loadData() }, 3000)
    }
  })

  stalledTimer = setInterval(() => {
    if (downloadingVersion.value && Date.now() - lastProgressTime > 120_000) {
      toast.warning(t('build.downloadStalled') || '下载似乎已停滞，请检查网络连接')
      lastProgressTime = Date.now()
    }
  }, 30_000)

  try {
    await api.downloadExportTemplate(version, mono)
    toast.success(t('build.templateDownloaded'))
    await loadData()
  } catch (e) {
    toast.error(e)
  } finally {
    downloadingVersion.value = null
    if (stalledTimer) clearTimeout(stalledTimer)
    unlisten()
  }
}

async function importTemplateFromFile(version: string, mono: boolean) {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Export Template', extensions: ['tpz', 'zip'] }],
      title: t('build.selectTemplateFile'),
    })
    if (!selected) return
    const filePath = typeof selected === 'string' ? selected : selected as unknown as string
    downloadingVersion.value = `${version}-import`
    try {
      await api.importExportTemplateFromFile(filePath, version, mono)
      toast.success(t('build.templateImported'))
      await loadData()
    } finally {
      downloadingVersion.value = null
    }
  } catch (e) {
    toast.error(e)
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
    toast.error(e)
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
    toast.error(e)
  }
}

async function exportPreset(preset: BuiltinExportPreset) {
  try {
    const json = await api.exportPresetToJson(preset as unknown as Record<string, unknown>)
    await navigator.clipboard.writeText(json)
    toast.success(t('build.presetExported') || '预设已复制到剪贴板')
  } catch (e) {
    toast.error(e)
  }
}

async function importPreset() {
  if (!presetProjectId.value || !importPresetJson.value.trim()) return
  try {
    await api.importPresetFromJson(presetProjectId.value, importPresetJson.value.trim())
    toast.success(t('build.presetImported') || '预设导入成功')
    importPresetJson.value = ''
  } catch (e) {
    toast.error(e)
  }
}

async function startBuild() {
  if (!selectedProjectId.value) {
    toast.error(t('build.selectProject'))
    return
  }
  const proj = projects.value.find(p => p.project_id === selectedProjectId.value)
  if (proj?.godot_version) {
    const templateInstalled = exportTemplates.value.some(
      t => t.version === proj.godot_version && t.installed
    )
    if (!templateInstalled) {
      toast.error(t('build.templateNotInstalled') || `导出模板未安装: Godot ${proj.godot_version}，请先在"导出模板"页下载`)
      return
    }
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
    toast.error(e)
  } finally {
    building.value = false
  }
}

async function retryBuild(record: BuildRecord) {
  selectedProjectId.value = record.project_id
  const platformMap: Record<string, ExportPlatform> = { 'Windows': 'Windows', 'macOS': 'MacOS', 'Linux': 'Linux', 'Web': 'Web', 'Android': 'Android', 'iOS': 'IOS' }
  selectedPlatform.value = platformMap[record.platform] || record.platform
  await startBuild()
}

async function generateCi() {
  if (!ciProjectId.value) {
    toast.error(t('build.selectProject'))
    return
  }
  generatingCi.value = true
  try {
    let config: string
    if (ciProvider.value === 'github-actions') {
      config = await api.generateGithubActions(ciProjectId.value, ciPlatforms.value, ciGodotVersion.value)
    } else {
      config = await api.generateGitlabCi(ciProjectId.value, ciPlatforms.value, ciGodotVersion.value)
    }
    generatedConfig.value = config
    toast.success(t('build.configGenerated'))
  } catch (e) {
    toast.error(e)
  } finally {
    generatingCi.value = false
  }
}

async function writeCiConfig() {
  if (!ciProjectId.value || !generatedConfig.value) return
  showWriteCiConfirm.value = true
}

async function confirmWriteCiConfig() {
  writingCi.value = true
  try {
    await api.writeCiConfig(ciProjectId.value!, ciProvider.value, generatedConfig.value!)
    toast.success(t('build.configWritten'))
  } catch (e) {
    toast.error(e)
  } finally {
    writingCi.value = false
  }
}

async function removeBuildRecord(buildId: string) {
  pendingDeleteRecordId.value = buildId
  showDeleteRecordConfirm.value = true
}

async function confirmDeleteRecord() {
  if (!pendingDeleteRecordId.value) return
  try {
    await api.deleteBuildRecord(pendingDeleteRecordId.value)
    buildRecords.value = buildRecords.value.filter(r => r.build_id !== pendingDeleteRecordId.value)
  } catch (e) {
    toast.error(e)
  } finally {
    pendingDeleteRecordId.value = null
  }
}

// 构建记录筛选
const buildFilterProject = ref('')

const filteredBuildRecords = computed(() => {
  if (!buildFilterProject.value) return buildRecords.value
  return buildRecords.value.filter(r => r.project_id === buildFilterProject.value)
})

const uniqueBuildProjects = computed(() => {
  const seen = new Map<string, string>()
  for (const r of buildRecords.value) {
    if (!seen.has(r.project_id)) {
      seen.set(r.project_id, r.project_name)
    }
  }
  return Array.from(seen.entries()).map(([id, name]) => ({ id, name }))
})

const showClearRecordsConfirm = ref(false)
const showDeleteTemplatesConfirm = ref(false)
const showWriteCiConfirm = ref(false)
const showDeleteRecordConfirm = ref(false)
const pendingDeleteRecordId = ref<string | null>(null)
const generatingCi = ref(false)
const writingCi = ref(false)

async function clearAllBuildRecords() {
  if (buildRecords.value.length === 0) return
  try {
    await api.clearAllBuildRecords()
    buildRecords.value = []
    toast.success(t('build.allRecordsCleared') || '构建记录已全部清除')
  } catch (e) {
    toast.error(e)
  }
}

async function downloadAllMissing() {
  const missing = exportTemplates.value.filter(t => !t.installed)
  if (missing.length === 0) return
  for (const tmpl of missing) {
    downloadingVersion.value = `${tmpl.version}-${tmpl.mono}`
    try {
      await api.downloadExportTemplate(tmpl.version, tmpl.mono)
    } catch (e) {
      toast.error(`${tmpl.version}: ${e}`)
    }
  }
  downloadingVersion.value = null
  toast.success(t('build.allDownloaded') || '全部下载完成')
  await loadData()
}

async function deleteAllInstalled() {
  const installed = exportTemplates.value.filter(t => t.installed)
  if (installed.length === 0) return
  for (const tmpl of installed) {
    try {
      await api.deleteExportTemplate(tmpl.version, tmpl.mono)
    } catch (e) {
      toast.error(`${tmpl.version}: ${e}`)
    }
  }
  toast.success(t('build.allDeleted') || '全部删除完成')
  await loadData()
}

async function startMcpServer() {
  try {
    await api.startMcpServer()
    mcpServerRunning.value = true
    toast.success(t('mcp.serverRunning'))
  } catch (e) {
    toast.error(e)
  }
}

async function stopMcpServer() {
  try {
    await api.stopMcpServer()
    mcpServerRunning.value = false
    toast.success(t('mcp.serverStopped') || 'MCP 服务器已停止')
  } catch (e) {
    toast.error(e)
  }
}

async function copyMcpStartCommand() {
  const cmd = mcpExePath.value.includes(' ') ? `& "${mcpExePath.value}"` : mcpExePath.value
  const ok = await copyToClipboard(cmd)
  if (ok) toast.success(t('mcp.startCmdCopied') || '启动命令已复制')
  else toast.error('Failed')
}

useDialogEscape(computed(() => !!deleteTarget.value))

onMounted(async () => {
  await Promise.all([loadData(), resolveMcpExePath(), loadMcpCapabilities()])
  unlistenProgress = await listen<BuildProgressPayload>('build-progress', (event) => {
    buildProgress.value = event.payload
    if (event.payload.stage === 'complete' || event.payload.stage === 'failed') {
      building.value = false
      setTimeout(() => { buildProgress.value = null }, 3000)
    }
  })
  unlistenDownloadProgress = await listen<DownloadProgressPayload>('export-template-download-progress', (event) => {
    downloadProgress.value = event.payload
    if (event.payload.stage === 'complete') {
      setTimeout(() => { downloadProgress.value = null; loadData() }, 1500)
    } else if (event.payload.stage === 'failed') {
      setTimeout(() => { downloadProgress.value = null; loadData() }, 3000)
    }
  })
  // 轮询 MCP 服务器状态
  const syncMcpState = async () => {
    try {
      mcpServerRunning.value = await api.isMcpServerRunning()
    } catch { /* ignore */ }
  }
  await syncMcpState()
  mcpPollTimer = setInterval(syncMcpState, 5000)
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenDownloadProgress?.()
  if (mcpPollTimer) clearInterval(mcpPollTimer)
})
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="px-6 pb-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">{{ t('build.title') }}</h1>
      <p class="text-sm text-gray-500 dark:text-content-muted mt-1">{{ t('build.subtitle') }}</p>
    </div>

    <div class="px-6 flex gap-1 border-b border-gray-200 dark:border-surface-border mb-4">
      <button
        v-for="tab in ([
          { key: 'export', label: t('build.exportConfig') || '导出配置' },
          { key: 'build', label: t('build.buildProject') },
          { key: 'mcp', label: t('build.mcpServer') || 'MCP Server' },
        ] as const)"
        :key="tab.key"
        class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
        :class="activeTab === tab.key
          ? 'border-primary-500 text-primary-600 dark:text-primary-400'
          : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-content-secondary dark:hover:text-content-primary'"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-6 pb-6">
      <div v-if="isLoading" class="flex items-center justify-center py-20">
        <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
      </div>
      <div v-else>
      <!-- Export Config Tab (merged: templates + presets) -->
      <div v-if="activeTab === 'export'">
        <div class="flex items-center justify-between mb-4">
          <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('build.exportTemplatesDesc') }}</p>
          <div v-if="exportTemplates.length > 0" class="flex items-center gap-2">
            <button
              v-if="exportTemplates.some(t => !t.installed)"
              class="text-xs text-primary-500 hover:text-primary-600 transition-colors"
              @click="downloadAllMissing"
            >
              {{ t('build.downloadAll') || '下载全部' }}
            </button>
            <button
              v-if="exportTemplates.some(t => t.installed)"
              class="text-xs text-red-400 hover:text-red-500 transition-colors"
              @click="showDeleteTemplatesConfirm = true"
            >
              {{ t('build.deleteAll') || '删除全部' }}
            </button>
          </div>
        </div>
        <div v-if="exportTemplates.length === 0" class="text-center py-12 text-gray-400">
          {{ t('build.notInstalled') }}
        </div>
        <div v-else class="grid gap-3">
          <div
            v-for="tmpl in exportTemplates"
            :key="tmpl.version + (tmpl.mono ? '-mono' : '')"
            class="flex items-center justify-between p-4 bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border"
          >
            <div class="flex items-center gap-3">
              <span
                class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
                :class="tmpl.installed
                  ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                  : 'bg-gray-100 text-gray-600 dark:bg-surface-layer dark:text-content-secondary'"
              >
                {{ tmpl.installed ? t('build.installed') : t('build.notInstalled') }}
              </span>
              <span class="font-medium text-gray-900 dark:text-content-primary">{{ tmpl.version }}</span>
              <span v-if="tmpl.mono" class="text-xs text-purple-600 dark:text-content-secondary bg-purple-50 dark:bg-surface-hover px-1.5 py-0.5 rounded">
                {{ t('build.mono') }}
              </span>
              <span v-if="tmpl.file_size" class="text-xs text-gray-400">{{ formatSize(tmpl.file_size) }}</span>
            </div>
            <div class="flex items-center gap-2">
              <button
                v-if="!tmpl.installed"
                class="px-3 py-1.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50"
                :disabled="downloadingVersion === `${tmpl.version}-${tmpl.mono}`"
                @click="downloadTemplate(tmpl.version, tmpl.mono)"
              >
                {{ downloadingVersion === `${tmpl.version}-${tmpl.mono}` ? t('build.downloading') : t('build.download') }}
              </button>
              <button
                v-if="!tmpl.installed"
                class="px-3 py-1.5 text-sm font-medium text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/10 rounded-lg transition-colors disabled:opacity-50"
                :disabled="downloadingVersion === `${tmpl.version}-import`"
                @click="importTemplateFromFile(tmpl.version, tmpl.mono)"
              >
                {{ t('build.importLocal') }}
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

        <div v-if="downloadProgress" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-xl p-4 mt-4">
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-5 w-5 border-2 border-primary-600 border-t-transparent flex-shrink-0" v-if="downloadProgress.stage !== 'complete'"></div>
            <svg v-else class="w-5 h-5 text-green-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <p class="text-sm font-medium text-primary-800 dark:text-primary-300">{{ downloadProgress.message }}</p>
                <span v-if="downloadProgress.stage !== 'complete'" class="text-xs font-medium text-primary-600 dark:text-primary-400 ml-2">{{ Math.round(downloadProgress.progress * 100) }}%</span>
              </div>
              <div v-if="downloadProgress.stage !== 'complete'" class="mt-2 w-full bg-primary-200 dark:bg-primary-800 rounded-full h-1.5">
                <div class="bg-primary-600 h-1.5 rounded-full transition-all" :style="{ width: (downloadProgress.progress * 100) + '%' }"></div>
              </div>
            </div>
          </div>
      </div>

      <!-- Export Presets Section -->
      <div class="mt-8 pt-6 border-t border-gray-200 dark:border-surface-border">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('build.presets') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">{{ t('build.presetsDesc') }}</p>
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('build.selectProject') }}</label>
          <ProjectSelector v-model="presetProjectId" :projects="projects" class="max-w-md" />
        </div>
        <h3 class="text-sm font-medium text-gray-700 dark:text-content-secondary mb-3">{{ t('build.builtinPresets') }}</h3>
        <div class="grid gap-3 sm:grid-cols-2">
          <div
            v-for="preset in builtinPresets"
            :key="preset.platform + preset.name"
            class="p-4 bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border"
          >
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-gray-900 dark:text-content-primary">{{ preset.name }}</span>
              <span class="text-xs text-gray-500 dark:text-content-muted">{{ preset.platform }}</span>
            </div>
            <p class="text-sm text-gray-500 dark:text-content-muted mb-3">{{ preset.description }}</p>
            <div class="flex items-center gap-2">
              <button
                class="px-3 py-1.5 text-sm font-medium text-primary-600 hover:text-primary-700 bg-primary-50 hover:bg-primary-100 dark:bg-primary-900/20 dark:text-primary-400 dark:hover:bg-primary-900/30 rounded-lg transition-colors"
                @click="applyPreset(preset)"
              >
                {{ t('build.applyPreset') }}
              </button>
              <button
                class="px-3 py-1.5 text-sm font-medium text-gray-600 hover:text-gray-700 bg-gray-50 hover:bg-gray-100 dark:bg-surface-layer dark:text-content-secondary dark:hover:bg-surface-border rounded-lg transition-colors"
                @click="exportPreset(preset)"
              >
                {{ t('build.exportPreset') || '导出' }}
              </button>
            </div>
          </div>
        </div>

        <div class="mt-6 p-4 bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border">
          <h3 class="text-sm font-medium text-gray-700 dark:text-content-secondary mb-3">{{ t('build.importPresetTitle') || '导入预设' }}</h3>
          <div class="flex gap-2">
            <input
              v-model="importPresetJson"
              type="text"
              :placeholder="t('build.importPresetPlaceholder') || '粘贴预设JSON'"
              class="flex-1 px-3 py-2 text-sm bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
            />
            <button
              class="px-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700 bg-primary-50 hover:bg-primary-100 dark:bg-primary-900/20 dark:text-primary-400 dark:hover:bg-primary-900/30 rounded-lg transition-colors disabled:opacity-50"
              :disabled="!importPresetJson.trim() || !presetProjectId"
              @click="importPreset"
            >
              {{ t('build.importPreset') || '导入' }}
            </button>
          </div>
        </div>
      </div>
      </div>

      <!-- Build Tab -->
      <div v-if="activeTab === 'build'">
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">{{ t('build.buildDesc') }}</p>
        <div class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border p-6 mb-6">
          <div class="grid gap-4 sm:grid-cols-3">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('build.selectProject') }}</label>
              <ProjectSelector v-model="selectedProjectId" :projects="projects" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('build.selectPlatform') }}</label>
              <select
                v-model="selectedPlatform"
                class="w-full px-3 py-2 bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-sm text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
              >
                <option v-for="opt in platformOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
              </select>
            </div>
            <div class="flex items-end">
              <button
                class="w-full px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50"
                :disabled="building || !selectedProjectId"
                @click="startBuild"
              >
                {{ building ? t('build.building') : t('build.startBuild') }}
              </button>
            </div>
          </div>
        </div>

        <div v-if="buildProgress" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-xl p-4 mb-6">
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-5 w-5 border-2 border-primary-600 border-t-transparent flex-shrink-0" v-if="buildProgress.stage !== 'complete' && buildProgress.stage !== 'failed'"></div>
            <svg v-else-if="buildProgress.stage === 'complete'" class="w-5 h-5 text-green-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            <svg v-else class="w-5 h-5 text-red-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <p class="text-sm font-medium text-primary-800 dark:text-primary-300">{{ buildProgress.message }}</p>
              </div>
              <div v-if="buildProgress.stage !== 'complete' && buildProgress.stage !== 'failed'" class="mt-2 w-full bg-primary-200 dark:bg-primary-800 rounded-full h-1.5 overflow-hidden">
                <div class="bg-primary-600 h-1.5 rounded-full animate-indeterminate-progress"></div>
              </div>
            </div>
          </div>
        </div>

        <h3 class="text-lg font-medium text-gray-900 dark:text-content-primary mb-3">{{ t('build.buildHistory') }}</h3>
        <div v-if="buildRecords.length > 0" class="flex items-center gap-3 mb-3">
          <select
            v-model="buildFilterProject"
            class="px-3 py-1.5 text-sm bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
          >
            <option value="">{{ t('build.allProjects') || '全部项目' }}</option>
            <option v-for="p in uniqueBuildProjects" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
          <div class="flex-1"></div>
          <button
            class="text-xs text-red-400 hover:text-red-500 transition-colors"
            @click="showClearRecordsConfirm = true"
          >
            {{ t('build.clearAll') || '全部清除' }}
          </button>
        </div>
        <div v-if="filteredBuildRecords.length === 0" class="text-center py-12 text-gray-400">
          {{ buildRecords.length === 0 ? t('build.noHistory') : (t('build.noMatchingRecords') || '无匹配记录') }}
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="record in filteredBuildRecords.slice().reverse()"
            :key="record.build_id"
            class="flex items-center justify-between p-4 bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border"
          >
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="font-medium text-gray-900 dark:text-content-primary">{{ record.project_name }}</span>
                <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-secondary">{{ record.platform }}</span>
                <span class="text-xs font-medium" :class="buildStatusClass(record.status)">{{ buildStatusText(record.status, t) }}</span>
              </div>
              <div class="text-xs text-gray-500 dark:text-content-muted flex gap-3">
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
                class="text-xs text-primary-500 hover:text-primary-600"
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

      <!-- CI/CD Section -->
      <div class="mt-8 pt-6 border-t border-gray-200 dark:border-surface-border">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('build.ciConfig') }}</h3>
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">{{ t('build.ciDesc') }}</p>
        <div class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border p-6 mb-6">
          <div class="grid gap-4 sm:grid-cols-2 mb-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('build.selectProject') }}</label>
              <ProjectSelector v-model="ciProjectId" :projects="projects" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('build.provider') }}</label>
              <select
                v-model="ciProvider"
                class="w-full px-3 py-2 bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-sm text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
              >
                <option value="github-actions">{{ t('build.githubActions') }}</option>
                <option value="gitlab-ci">{{ t('build.gitlabCi') }}</option>
              </select>
            </div>
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('build.godotVersion') }}</label>
            <input
              v-model="ciGodotVersion"
              type="text"
              class="w-full max-w-xs px-3 py-2 bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-sm text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
              :class="{ 'border-red-400 dark:border-red-500': ciGodotVersion && !isValidGodotVersion }"
              placeholder="4.4.1"
            />
            <p v-if="ciGodotVersion && !isValidGodotVersion" class="mt-1 text-xs text-red-500">{{ t('build.invalidVersion') || '版本号格式不正确，如 4.4.1' }}</p>
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('build.selectPlatforms') }}</label>
            <div class="flex flex-wrap gap-2">
              <label
                v-for="opt in ciPlatformOptions"
                :key="opt.value"
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-sm cursor-pointer transition-colors"
                :class="ciPlatforms.includes(opt.value)
                  ? 'border-primary-500 bg-primary-50 text-primary-700 dark:bg-primary-900/20 dark:text-primary-400'
                  : 'border-gray-300 dark:border-surface-border text-gray-600 dark:text-content-secondary'"
              >
                <input type="checkbox" :value="opt.value" v-model="ciPlatforms" class="sr-only" />
                {{ opt.label }}
              </label>
            </div>
          </div>
          <div class="flex gap-2">
            <button
              class="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50"
              :disabled="!ciProjectId || ciPlatforms.length === 0 || !isValidGodotVersion || generatingCi"
              @click="generateCi"
            >
              {{ generatingCi ? t('build.generating') || '生成中...' : t('build.generate') }}
            </button>
          </div>
        </div>

        <div v-if="generatedConfig" class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border p-6">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-medium text-gray-700 dark:text-content-secondary">
              {{ ciProvider === 'github-actions' ? '.github/workflows/build.yml' : '.gitlab-ci.yml' }}
            </h3>
            <div class="flex items-center gap-2">
              <button
                class="px-3 py-1.5 text-sm font-medium text-gray-600 hover:text-gray-700 bg-gray-50 hover:bg-gray-100 dark:bg-surface-layer dark:text-content-secondary dark:hover:bg-surface-hover rounded-lg transition-colors"
                @click="copyToClipboard(generatedConfig).then(ok => ok ? toast.success(t('mcp.configCopied')) : toast.error('Failed'))"
              >
                {{ t('build.copyConfig') || '复制' }}
              </button>
              <button
                class="px-3 py-1.5 text-sm font-medium text-green-600 hover:text-green-700 bg-green-50 hover:bg-green-100 dark:bg-green-900/20 dark:text-green-400 dark:hover:bg-green-900/30 rounded-lg transition-colors disabled:opacity-50"
                :disabled="writingCi"
                @click="writeCiConfig"
              >
                {{ writingCi ? t('build.writing') || '写入中...' : t('build.writeConfig') }}
              </button>
            </div>
          </div>
          <textarea
            v-model="generatedConfig"
            class="w-full bg-gray-50 dark:bg-surface-layer rounded-lg p-4 text-xs text-gray-800 dark:text-content-secondary font-mono resize-y min-h-[200px] max-h-96 focus:ring-2 focus:ring-primary-500 outline-none"
            spellcheck="false"
          ></textarea>
        </div>
      </div>
      </div>

      <!-- MCP Server Tab -->
      <div v-if="activeTab === 'mcp'">
        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">{{ t('mcp.subtitle') }}</p>

        <div class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border p-6 mb-6">
          <div class="flex items-center justify-between mb-4">
            <div>
              <h3 class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('mcp.title') }}</h3>
              <p class="text-xs mt-1" :class="mcpServerRunning ? 'text-green-500' : 'text-gray-400'">
                {{ mcpServerRunning ? t('mcp.serverRunning') : t('mcp.serverStopped') }}
              </p>
            </div>
            <div class="flex items-center gap-2">
              <button
                v-if="!mcpServerRunning"
                class="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
                @click="startMcpServer"
              >
                {{ t('mcp.startServer') }}
              </button>
              <button
                v-if="mcpServerRunning"
                class="px-4 py-2 text-sm font-medium text-red-600 hover:text-red-700 bg-red-50 hover:bg-red-100 dark:bg-red-900/20 dark:text-red-400 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                @click="stopMcpServer"
              >
                {{ t('mcp.stopServer') || '停止服务' }}
              </button>
            </div>
          </div>

          <div class="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg mb-4">
            <p class="text-xs text-amber-700 dark:text-amber-300">{{ t('mcp.standaloneHint') || 'MCP Server 通过 stdio 协议通信，需要由 AI 编程工具（如 Claude Desktop、Cursor）独立启动。上方的启动/停止按钮仅供内部测试使用。' }}</p>
          </div>

          <div v-if="mcpExeNotFound" class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg mb-4">
            <p class="text-xs text-red-700 dark:text-red-300">{{ t('mcp.exeNotFound') || 'MCP Server 可执行文件未找到，请先构建项目或检查安装路径。' }}</p>
          </div>

          <div class="flex items-center gap-2 mb-3">
            <span class="text-xs text-gray-500 dark:text-content-muted">{{ t('mcp.exePath') || '可执行文件路径' }}:</span>
            <code class="text-xs bg-gray-100 dark:bg-surface-layer px-2 py-0.5 rounded break-all">{{ mcpExePath }}</code>
            <button
              class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400"
              @click="copyToClipboard(mcpExePath).then(ok => ok ? toast.success(t('mcp.pathCopied') || '路径已复制') : toast.error('Failed'))"
            >
              {{ t('mcp.copyPath') || '复制' }}
            </button>
          </div>

          <button
            class="px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-content-secondary bg-gray-100 dark:bg-surface-layer hover:bg-gray-200 dark:hover:bg-surface-hover rounded-lg transition-colors"
            @click="copyMcpStartCommand"
          >
            {{ t('mcp.copyStartCommand') || '复制终端启动命令' }}
          </button>
        </div>

        <div class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border p-6 mb-6">
          <h3 class="text-sm font-medium text-gray-900 dark:text-content-primary mb-3">{{ t('mcp.capabilities') || '可用能力' }}</h3>
          <div class="flex gap-4 mb-4">
            <button
              v-for="section in [
                { key: 'tools', label: t('mcp.tools') || 'Tools', count: mcpCapabilities?.tools_count ?? 0 },
                { key: 'resources', label: t('mcp.resources') || 'Resources', count: mcpCapabilities?.resources_count ?? 0 },
                { key: 'prompts', label: t('mcp.prompts') || 'Prompts', count: mcpCapabilities?.prompts_count ?? 0 }
              ]"
              :key="section.key"
              class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors cursor-pointer"
              :class="mcpExpandedSection === section.key
                ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-400 border border-primary-200 dark:border-primary-800'
                : 'bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary border border-transparent hover:bg-gray-100 dark:hover:bg-surface-hover'"
              @click="mcpExpandedSection = mcpExpandedSection === section.key ? null : section.key as any"
            >
              <span class="font-medium">{{ section.label }}</span>
              <span class="text-xs px-1.5 py-0.5 rounded-full"
                :class="mcpExpandedSection === section.key ? 'bg-primary-200 dark:bg-primary-800 text-primary-800 dark:text-primary-200' : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400'">
                {{ section.count }}
              </span>
            </button>
          </div>

          <div v-if="mcpExpandedSection === 'tools' && mcpCapabilities" class="space-y-1">
            <div v-for="tool in mcpCapabilities.tools" :key="tool.name" class="flex items-start gap-3 p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover">
              <code class="text-xs font-mono text-primary-600 dark:text-primary-400 whitespace-nowrap mt-0.5">{{ tool.name }}</code>
              <span class="text-xs text-gray-500 dark:text-content-muted">{{ tool.description }}</span>
            </div>
          </div>
          <div v-if="mcpExpandedSection === 'resources' && mcpCapabilities" class="space-y-1">
            <div v-for="res in mcpCapabilities.resources" :key="res.uri" class="flex items-start gap-3 p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover">
              <code class="text-xs font-mono text-green-600 dark:text-green-400 whitespace-nowrap mt-0.5">{{ res.uri }}</code>
              <span class="text-xs text-gray-500 dark:text-content-muted">{{ res.description }}</span>
            </div>
          </div>
          <div v-if="mcpExpandedSection === 'prompts' && mcpCapabilities" class="space-y-1">
            <div v-for="prompt in mcpCapabilities.prompts" :key="prompt.name" class="flex items-start gap-3 p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover">
              <code class="text-xs font-mono text-amber-600 dark:text-amber-400 whitespace-nowrap mt-0.5">{{ prompt.name }}</code>
              <span class="text-xs text-gray-500 dark:text-content-muted">{{ prompt.description }}</span>
            </div>
          </div>
        </div>

        <div class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border p-6 mb-6">
          <h3 class="text-sm font-medium text-gray-900 dark:text-content-primary mb-3">{{ t('mcp.clientConfig') || '客户端配置' }}</h3>
          <div class="flex flex-wrap gap-2 mb-4">
            <button
              v-for="client in mcpClients"
              :key="client.key"
              class="px-3 py-1.5 text-xs font-medium rounded-lg transition-colors"
              :class="mcpSelectedClient === client.key
                ? 'bg-primary-600 text-white'
                : 'bg-gray-100 dark:bg-surface-layer text-gray-700 dark:text-content-secondary hover:bg-gray-200 dark:hover:bg-surface-hover'"
              @click="mcpSelectedClient = client.key"
            >
              {{ client.title }}
            </button>
          </div>
          <div v-for="client in mcpClients" :key="client.key">
            <div v-if="mcpSelectedClient === client.key">
              <p class="text-xs text-gray-500 dark:text-content-muted mb-2">{{ client.desc }}</p>
              <p class="text-xs text-gray-400 dark:text-content-muted mb-2">{{ t('mcp.configFilePath') || '配置文件路径' }}: <code class="text-xs">{{ client.configPath }}</code></p>
              <div class="relative">
                <pre class="bg-gray-50 dark:bg-surface-layer rounded-lg p-3 text-xs text-gray-800 dark:text-content-secondary overflow-x-auto max-h-48 overflow-y-auto">{{ mcpConfig }}</pre>
                <button
                  class="absolute top-2 right-2 px-2 py-1 text-xs font-medium text-primary-600 hover:text-primary-700 bg-white dark:bg-surface-card hover:bg-gray-50 dark:hover:bg-surface-hover rounded border border-gray-200 dark:border-surface-border transition-colors"
                  @click="copyToClipboard(mcpConfig).then(ok => ok ? toast.success(t('mcp.configCopied')) : toast.error('Failed to copy'))"
                >
                  {{ t('mcp.copyConfig') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border p-6">
          <h3 class="text-sm font-medium text-gray-900 dark:text-content-primary mb-3">{{ t('mcp.usageGuide') || '使用指南' }}</h3>
          <ol class="space-y-2 text-xs text-gray-600 dark:text-content-secondary">
            <li class="flex gap-2"><span class="font-medium text-primary-600 dark:text-primary-400">1.</span> {{ t('mcp.step1') || '点击上方"复制配置"按钮，复制 JSON 配置' }}</li>
            <li class="flex gap-2"><span class="font-medium text-primary-600 dark:text-primary-400">2.</span> {{ t('mcp.step2') || '打开 AI 编程工具的 MCP 配置文件（见上方路径）' }}</li>
            <li class="flex gap-2"><span class="font-medium text-primary-600 dark:text-primary-400">3.</span> {{ t('mcp.step3') || '将 JSON 配置粘贴到配置文件中并保存' }}</li>
            <li class="flex gap-2"><span class="font-medium text-primary-600 dark:text-primary-400">4.</span> {{ t('mcp.step4') || '重启 AI 编程工具' }}</li>
            <li class="flex gap-2"><span class="font-medium text-primary-600 dark:text-primary-400">5.</span> {{ t('mcp.step5') || '在对话中输入"帮我检查项目环境"测试 MCP 是否工作' }}</li>
          </ol>
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
  </div>

  <ConfirmDialog
    v-model="showClearRecordsConfirm"
    :title="t('build.clearAll') || '全部清除'"
    :description="t('build.clearAllConfirm') || '确定要清除所有构建记录吗？此操作不可撤销。'"
    :confirm-text="t('build.clearAll') || '全部清除'"
    confirm-color="red"
    @confirm="clearAllBuildRecords"
  />

  <ConfirmDialog
    v-model="showDeleteTemplatesConfirm"
    :title="t('build.deleteAll') || '删除全部'"
    :description="t('build.deleteAllConfirm') || '确定要删除所有已安装的导出模板吗？此操作不可撤销。'"
    :confirm-text="t('build.deleteAll') || '删除全部'"
    confirm-color="red"
    @confirm="deleteAllInstalled"
  />

  <ConfirmDialog
    v-model="showWriteCiConfirm"
    :title="t('build.confirmWriteCiTitle') || '写入 CI 配置'"
    :description="t('build.confirmWriteCiDesc') || '将覆盖项目中已有的 CI 配置文件，确定要继续吗？'"
    :confirm-text="t('common.confirm') || '确认'"
    confirm-color="green"
    @confirm="confirmWriteCiConfig"
  />

  <ConfirmDialog
    v-model="showDeleteRecordConfirm"
    :title="t('build.deleteRecord') || '删除记录'"
    :description="t('build.deleteRecordConfirm') || '确定要删除这条构建记录吗？'"
    :confirm-text="t('build.delete') || '删除'"
    confirm-color="red"
    @confirm="confirmDeleteRecord"
  />
</template>

<style scoped>
@keyframes indeterminate-progress {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(400%); }
}
.animate-indeterminate-progress {
  animation: indeterminate-progress 1.5s ease-in-out infinite;
  width: 25%;
}
</style>
