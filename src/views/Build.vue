<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue'
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

const toast = useToast()
const { t } = useI18n()
const { openInFileManager } = useFileManager()

const activeTab = ref<'export' | 'build'>('export')
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
const buildLogs = ref<Array<{ line: string; stream: string }>>([])
const logPanelRef = ref<HTMLElement | null>(null)

let unlistenProgress: UnlistenFn | null = null
let unlistenDownloadProgress: UnlistenFn | null = null
let unlistenBuildLog: UnlistenFn | null = null

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
      toast.warning(t('build.downloadStalled'))
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
    toast.success(t('build.presetExported'))
  } catch (e) {
    toast.error(e)
  }
}

async function importPreset() {
  if (!presetProjectId.value || !importPresetJson.value.trim()) return
  try {
    await api.importPresetFromJson(presetProjectId.value, importPresetJson.value.trim())
    toast.success(t('build.presetImported'))
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

async function handleCancelBuild() {
  try {
    const cancelled = await api.cancelBuild()
    if (cancelled) {
      toast.success(t('build.buildCancelled'))
    } else {
      toast.warning(t('build.noActiveBuild'))
    }
    building.value = false
  } catch (e) {
    toast.error(e)
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
    toast.success(t('build.allRecordsCleared'))
  } catch (e) {
    toast.error(e)
  }
}

async function downloadAllMissing() {
  const missing = exportTemplates.value.filter(t => !t.installed)
  if (missing.length === 0) return
  let failCount = 0
  for (const tmpl of missing) {
    downloadingVersion.value = `${tmpl.version}-${tmpl.mono}`
    try {
      await api.downloadExportTemplate(tmpl.version, tmpl.mono)
    } catch (e) {
      toast.error(`${tmpl.version}: ${e}`)
      failCount++
    }
  }
  downloadingVersion.value = null
  if (failCount === 0) {
    toast.success(t('build.allDownloaded'))
  } else {
    toast.warning(t('build.partialFailed', { failed: failCount, total: missing.length }))
  }
  await loadData()
}

async function deleteAllInstalled() {
  const installed = exportTemplates.value.filter(t => t.installed)
  if (installed.length === 0) return
  let failCount = 0
  for (const tmpl of installed) {
    try {
      await api.deleteExportTemplate(tmpl.version, tmpl.mono)
    } catch (e) {
      toast.error(`${tmpl.version}: ${e}`)
      failCount++
    }
  }
  if (failCount === 0) {
    toast.success(t('build.allDeleted'))
  } else {
    toast.warning(t('build.partialFailed', { failed: failCount, total: installed.length }))
  }
  await loadData()
}

useDialogEscape(computed(() => !!deleteTarget.value))

onMounted(async () => {
  await loadData()
  unlistenProgress = await listen<BuildProgressPayload>('build-progress', (event) => {
    buildProgress.value = event.payload
    if (event.payload.stage === 'starting') {
      buildLogs.value = []
    }
    if (event.payload.stage === 'complete' || event.payload.stage === 'failed') {
      building.value = false
    }
  })
  unlistenBuildLog = await listen<{ build_id: string; line: string; stream: string }>('build-log', (event) => {
    buildLogs.value.push({ line: event.payload.line, stream: event.payload.stream })
    if (buildLogs.value.length > 5000) {
      buildLogs.value = buildLogs.value.slice(-3000)
    }
    nextTick(() => {
      if (logPanelRef.value) {
        logPanelRef.value.scrollTop = logPanelRef.value.scrollHeight
      }
    })
  })
  unlistenDownloadProgress = await listen<DownloadProgressPayload>('export-template-download-progress', (event) => {
    downloadProgress.value = event.payload
    if (event.payload.stage === 'complete') {
      setTimeout(() => { downloadProgress.value = null; loadData() }, 1500)
    } else if (event.payload.stage === 'failed') {
      setTimeout(() => { downloadProgress.value = null; loadData() }, 3000)
    }
  })
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenDownloadProgress?.()
  unlistenBuildLog?.()
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
          { key: 'export', label: t('build.exportConfig') },
          { key: 'build', label: t('build.buildProject') },
        ] as const)"
        :key="tab.key"
        class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
        :class="activeTab === tab.key
          ? 'border-primary-500 text-primary-600 dark:text-brand-primary'
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
              {{ t('build.downloadAll') }}
            </button>
            <button
              v-if="exportTemplates.some(t => t.installed)"
              class="text-xs text-red-400 hover:text-red-500 transition-colors"
              @click="showDeleteTemplatesConfirm = true"
            >
              {{ t('build.deleteAll') }}
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
                class="px-3 py-1.5 text-sm font-medium text-primary-600 dark:text-brand-primary hover:bg-primary-50 dark:hover:bg-surface-hover rounded-lg transition-colors disabled:opacity-50"
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

        <div v-if="downloadProgress" class="bg-primary-50 dark:bg-surface-hover border border-primary-200 dark:border-surface-border rounded-xl p-4 mt-4">
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-5 w-5 border-2 border-primary-600 border-t-transparent flex-shrink-0" v-if="downloadProgress.stage !== 'complete'"></div>
            <svg v-else class="w-5 h-5 text-green-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <p class="text-sm font-medium text-primary-800 dark:text-content-secondary">{{ downloadProgress.message }}</p>
                <span v-if="downloadProgress.stage !== 'complete'" class="text-xs font-medium text-primary-600 dark:text-brand-primary ml-2">{{ Math.round(downloadProgress.progress * 100) }}%</span>
              </div>
              <div v-if="downloadProgress.stage !== 'complete'" class="mt-2 w-full bg-surface-border dark:bg-surface-border rounded-full h-1.5">
                <div class="bg-brand-primary h-1.5 rounded-full transition-all" :style="{ width: (downloadProgress.progress * 100) + '%' }"></div>
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
                class="px-3 py-1.5 text-sm font-medium text-primary-600 hover:text-primary-700 bg-primary-50 hover:bg-primary-100 dark:bg-surface-hover dark:text-brand-primary dark:hover:bg-surface-hover rounded-lg transition-colors"
                @click="applyPreset(preset)"
              >
                {{ t('build.applyPreset') }}
              </button>
              <button
                class="px-3 py-1.5 text-sm font-medium text-gray-600 hover:text-gray-700 bg-gray-50 hover:bg-gray-100 dark:bg-surface-layer dark:text-content-secondary dark:hover:bg-surface-border rounded-lg transition-colors"
                @click="exportPreset(preset)"
              >
                {{ t('build.exportPreset') }}
              </button>
            </div>
          </div>
        </div>

        <div class="mt-6 p-4 bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border">
          <h3 class="text-sm font-medium text-gray-700 dark:text-content-secondary mb-3">{{ t('build.importPresetTitle') }}</h3>
          <div class="flex gap-2">
            <input
              v-model="importPresetJson"
              type="text"
              :placeholder="t('build.importPresetPlaceholder')"
              class="flex-1 px-3 py-2 text-sm bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
            />
            <button
              class="px-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700 bg-primary-50 hover:bg-primary-100 dark:bg-surface-hover dark:text-brand-primary dark:hover:bg-surface-hover rounded-lg transition-colors disabled:opacity-50"
              :disabled="!importPresetJson.trim() || !presetProjectId"
              @click="importPreset"
            >
              {{ t('build.importPreset') }}
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
            <div class="flex items-end gap-2">
              <button
                class="flex-1 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50"
                :disabled="building || !selectedProjectId"
                @click="startBuild"
              >
                {{ building ? t('build.building') : t('build.startBuild') }}
              </button>
              <button
                v-if="building"
                @click="handleCancelBuild"
                class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
              >
                {{ t('build.cancelBuild') }}
              </button>
            </div>
          </div>
        </div>

        <div v-if="buildProgress" class="bg-primary-50 dark:bg-surface-hover border border-primary-200 dark:border-surface-border rounded-xl p-4 mb-6">
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-5 w-5 border-2 border-primary-600 border-t-transparent flex-shrink-0" v-if="buildProgress.stage !== 'complete' && buildProgress.stage !== 'failed'"></div>
            <svg v-else-if="buildProgress.stage === 'complete'" class="w-5 h-5 text-green-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            <svg v-else class="w-5 h-5 text-red-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <p class="text-sm font-medium text-primary-800 dark:text-content-secondary">{{ buildProgress.message }}</p>
              </div>
              <div v-if="buildProgress.stage !== 'complete' && buildProgress.stage !== 'failed'" class="mt-2 w-full bg-surface-border dark:bg-surface-border rounded-full h-1.5 overflow-hidden">
                <div class="bg-brand-primary h-1.5 rounded-full animate-indeterminate-progress"></div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="buildLogs.length > 0" class="mb-6">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('build.buildLog') }}</h3>
            <button @click="buildLogs = []" class="text-xs text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-content-secondary">{{ t('common.clear') }}</button>
          </div>
          <div ref="logPanelRef" class="bg-surface-layer dark:bg-surface-base border border-surface-border rounded-lg p-3 max-h-80 overflow-y-auto font-mono text-xs leading-relaxed">
            <div v-for="(log, i) in buildLogs" :key="i" :class="log.stream === 'stderr' ? 'text-red-400' : 'text-content-secondary'">{{ log.line }}</div>
          </div>
        </div>

        <h3 class="text-lg font-medium text-gray-900 dark:text-content-primary mb-3">{{ t('build.buildHistory') }}</h3>
        <div v-if="buildRecords.length > 0" class="flex items-center gap-3 mb-3">
          <select
            v-model="buildFilterProject"
            class="px-3 py-1.5 text-sm bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
          >
            <option value="">{{ t('build.allProjects') }}</option>
            <option v-for="p in uniqueBuildProjects" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
          <div class="flex-1"></div>
          <button
            class="text-xs text-red-400 hover:text-red-500 transition-colors"
            @click="showClearRecordsConfirm = true"
          >
            {{ t('build.clearAll') }}
          </button>
        </div>
        <div v-if="filteredBuildRecords.length === 0" class="text-center py-12 text-gray-400">
          {{ buildRecords.length === 0 ? t('build.noHistory') : t('build.noMatchingRecords') }}
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
                {{ t('build.retry') }}
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
            <p v-if="ciGodotVersion && !isValidGodotVersion" class="mt-1 text-xs text-red-500">{{ t('build.invalidVersion') }}</p>
          </div>
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-2">{{ t('build.selectPlatforms') }}</label>
            <div class="flex flex-wrap gap-2">
              <label
                v-for="opt in ciPlatformOptions"
                :key="opt.value"
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-sm cursor-pointer transition-colors"
                :class="ciPlatforms.includes(opt.value)
                  ? 'border-primary-500 bg-primary-50 text-primary-700 dark:bg-surface-hover dark:text-brand-primary'
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
              {{ generatingCi ? t('build.generating') : t('build.generate') }}
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
                @click="copyToClipboard(generatedConfig).then(ok => ok ? toast.success(t('build.configCopied')) : toast.error('Failed'))"
              >
                {{ t('build.copyConfig') }}
              </button>
              <button
                class="px-3 py-1.5 text-sm font-medium text-green-600 hover:text-green-700 bg-green-50 hover:bg-green-100 dark:bg-green-900/20 dark:text-green-400 dark:hover:bg-green-900/30 rounded-lg transition-colors disabled:opacity-50"
                :disabled="writingCi"
                @click="writeCiConfig"
              >
                {{ writingCi ? t('build.writing') : t('build.writeConfig') }}
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
    :title="t('build.clearAll')"
    :description="t('build.clearAllConfirm')"
    :confirm-text="t('build.clearAll')"
    confirm-color="red"
    @confirm="clearAllBuildRecords"
  />

  <ConfirmDialog
    v-model="showDeleteTemplatesConfirm"
    :title="t('build.deleteAll')"
    :description="t('build.deleteAllConfirm')"
    :confirm-text="t('build.deleteAll')"
    confirm-color="red"
    @confirm="deleteAllInstalled"
  />

  <ConfirmDialog
    v-model="showWriteCiConfirm"
    :title="t('build.confirmWriteCiTitle')"
    :description="t('build.confirmWriteCiDesc')"
    :confirm-text="t('common.confirm')"
    confirm-color="green"
    @confirm="confirmWriteCiConfig"
  />

  <ConfirmDialog
    v-model="showDeleteRecordConfirm"
    :title="t('build.deleteRecord')"
    :description="t('build.deleteRecordConfirm')"
    :confirm-text="t('build.delete')"
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
