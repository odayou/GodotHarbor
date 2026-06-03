<script setup lang="ts">
import { ref, onMounted, onActivated, computed, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { Template, TemplateCategory, TemplateInstantiationProgress, Project } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useEngineLauncher } from '@/composables/useEngineLauncher'
import { useFileManager } from '@/composables/useFileManager'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { isOnline } from '@/composables/useNetworkStatus'
import EmptyState from '@/components/EmptyState.vue'
import SkeletonList from '@/components/SkeletonList.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ProjectSelector from '@/components/ProjectSelector.vue'

const toast = useToast()
const { t } = useI18n()
const {
  openProjectWithEngine,
  showEngineSelectDialog,
  engineSelectProject,
  matchedEngines,
  isLoadingEngines,
  isLaunching,
  launchWithEngine,
  closeEngineSelectDialog,
  getMatchLevelClass,
  getMatchLevelLabel,
  getMatchLevelDesc,
} = useEngineLauncher()
const { openInFileManager } = useFileManager()

const templates = ref<Template[]>([])
const isLoading = ref(true)
const loadError = ref<string | null>(null)
const categoryFilter = ref<TemplateCategory | 'all'>('all')
const searchQuery = ref('')

const showDetailDialog = ref(false)
const selectedTemplate = ref<Template | null>(null)

const showCreateDialog = ref(false)
const createProjectName = ref('')
const createTargetDir = ref('')
const enableMobileSupport = ref(false)
const isCreating = ref(false)
const createProgress = ref<TemplateInstantiationProgress | null>(null)
const projectNameError = ref('')

const isValidProjectName = computed(() => {
  const name = createProjectName.value.trim()
  if (!name) return false
  if (/[<>:"/\\|?*]/.test(name)) return false
  if (name.startsWith('.') || name.endsWith('.')) return false
  if (name.length > 200) return false
  return true
})

const validateProjectName = () => {
  const name = createProjectName.value.trim()
  if (!name) {
    projectNameError.value = ''
    return
  }
  if (/[<>:"/\\|?*]/.test(name)) {
    projectNameError.value = t('templates.invalidChars') || '项目名包含非法字符'
  } else if (name.startsWith('.') || name.endsWith('.')) {
    projectNameError.value = t('templates.invalidStartEnd') || '项目名不能以点号开头或结尾'
  } else {
    projectNameError.value = ''
  }
}

const showImportDialog = ref(false)
const importUrl = ref('')
const isImporting = ref(false)

const showGenerateFromProjectDialog = ref(false)
const generateProjectId = ref('')
const generateTemplateName = ref('')
const generateCategory = ref<TemplateCategory>('Custom')
const isGenerating = ref(false)
const projects = ref<Project[]>([])

const showDeleteConfirm = ref(false)
const deleteTargetId = ref('')

// 右键菜单
const contextMenuTemplate = ref<Template | null>(null)
const contextMenuPos = ref({ x: 0, y: 0 })
const showContextMenu = ref(false)

const onTemplateContextMenu = (e: MouseEvent, tpl: Template) => {
  e.preventDefault()
  contextMenuTemplate.value = tpl
  contextMenuPos.value = { x: e.clientX, y: e.clientY }
  showContextMenu.value = true
}

const closeContextMenu = () => {
  showContextMenu.value = false
  contextMenuTemplate.value = null
}

useDialogEscape(showDetailDialog)
useDialogEscape(showCreateDialog)
useDialogEscape(showImportDialog)
useDialogEscape(showGenerateFromProjectDialog)

let unlistenProgress: UnlistenFn | null = null

onMounted(async () => {
  const [, projectList] = await Promise.all([
    loadTemplates(),
    api.getProjects().catch(() => [] as Project[])
  ])
  projects.value = projectList
  unlistenProgress = await listen('template-instantiation-progress', (event) => {
    createProgress.value = event.payload as TemplateInstantiationProgress
  })
})

onActivated(() => {
  loadTemplates()
})

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress()
  }
})

const loadTemplates = async () => {
  isLoading.value = true
  loadError.value = null
  try {
    await api.ensureBuiltinTemplates()
    templates.value = await api.listHubTemplates()
  } catch (e: any) {
    loadError.value = e?.toString() || 'Failed to load templates'
  } finally {
    isLoading.value = false
  }
}

const filteredTemplates = computed(() => {
  return templates.value.filter(tpl => {
    const matchesCategory = categoryFilter.value === 'all' || tpl.category === categoryFilter.value
    const matchesSearch = searchQuery.value === '' ||
      tpl.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      tpl.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      tpl.tags.some(tag => tag.toLowerCase().includes(searchQuery.value.toLowerCase()))
    return matchesCategory && matchesSearch
  })
})

const categories = computed(() => {
  const cats = new Set<TemplateCategory>(templates.value.map(t => t.category))
  return ['all', ...Array.from(cats)] as const
})

const categoryIcon = (cat: TemplateCategory | 'all') => {
  switch (cat) {
    case 'all': return '📋'
    case 'Starter2D': return '🎮'
    case 'Starter3D': return '🌐'
    case 'RPG': return '⚔️'
    case 'Platformer': return '🏃'
    case 'Multiplayer': return '👥'
    case 'Mobile': return '📱'
    case 'Blank': return '📄'
    case 'Custom': return '🔧'
    default: return '📁'
  }
}

const openDetail = (tpl: Template) => {
  selectedTemplate.value = tpl
  showDetailDialog.value = true
}

const openCreateDialog = async (tpl: Template) => {
  selectedTemplate.value = tpl
  createProjectName.value = ''
  createProgress.value = null
  projectNameError.value = ''
  enableMobileSupport.value = false
  if (!createTargetDir.value) {
    try {
      const paths = await api.getStoragePaths()
      const docsDir = paths.app_data_dir.replace(/[/\\]GodotHarbor[/\\]?$/, '')
      createTargetDir.value = docsDir
    } catch {
      createTargetDir.value = ''
    }
  }
  showCreateDialog.value = true
}

const selectTargetDir = async () => {
  const selected = await open({ directory: true, multiple: false, title: t('projects.selectDir') || 'Select Directory' })
  if (selected) {
    createTargetDir.value = selected as string
  }
}

const lastCreatedProjectId = ref('')
const lastCreatedProject = ref<Project | null>(null)

const handleCreate = async () => {
  if (!selectedTemplate.value || !createProjectName.value.trim() || !createTargetDir.value.trim()) return
  if (!isValidProjectName.value) return

  isCreating.value = true
  createProgress.value = null
  try {
    const result = await api.instantiateTemplate(
      selectedTemplate.value.template_id,
      createProjectName.value.trim(),
      createTargetDir.value.trim(),
      enableMobileSupport.value
    )
    showCreateDialog.value = false
    lastCreatedProjectId.value = result.project_id

    // 获取创建的项目信息用于成功提示
    try {
      const allProjects = await api.getProjects()
      lastCreatedProject.value = allProjects.find(p => p.project_id === result.project_id) || null
    } catch {
      lastCreatedProject.value = null
    }

    if (result.failed_plugins.length > 0) {
      const details = result.failed_plugins.join('\n')
      toast.warning(`${t('templates.createSuccess')} (${result.failed_plugins.length} ${t('templates.partialFailed') || '项未完成'}):\n${details}`, 8000)
    } else {
      toast.success(t('templates.createSuccess'))
    }

    if (result.engine_installed) {
      try {
        const projects = await api.getProjects()
        const project = projects.find(p => p.project_id === result.project_id)
        if (project) {
          await openProjectWithEngine(project)
        }
      } catch {
        // 自动打开失败不影响创建结果
      }
    }
  } catch (e: any) {
    toast.error(`${t('templates.createFailed')}: ${e?.toString() || e}`)
  } finally {
    isCreating.value = false
  }
}

const handleImport = async () => {
  if (!importUrl.value.trim()) return
  if (!isOnline.value) {
    toast.error(t('common.offlineError') || '网络不可用')
    return
  }
  isImporting.value = true
  try {
    await api.importTemplateFromUrl(importUrl.value.trim())
    toast.success(t('templates.importSuccess'))
    showImportDialog.value = false
    importUrl.value = ''
    await loadTemplates()
  } catch (e: any) {
    toast.error(`${t('templates.importFailed') || 'Import failed'}: ${e?.toString() || e}`)
  } finally {
    isImporting.value = false
  }
}

const handleDelete = async () => {
  try {
    await api.deleteHubTemplate(deleteTargetId.value)
    toast.success(t('templates.deleteSuccess') || t('templates.saveSuccess'))
    await loadTemplates()
  } catch (e: any) {
    toast.error(`Delete failed: ${e?.toString() || e}`)
  }
  showDeleteConfirm.value = false
  deleteTargetId.value = ''
}

const handleGenerateFromProject = async () => {
  if (!generateProjectId.value || !generateTemplateName.value.trim()) return
  isGenerating.value = true
  try {
    // 前端使用 PascalCase 分类名，后端期望 snake_case
    const categoryMap: Record<string, string> = {
      Starter2D: 'starter_2d',
      Starter3D: 'starter_3d',
      RPG: 'rpg',
      Platformer: 'platformer',
      Multiplayer: 'multiplayer',
      Mobile: 'mobile',
      Blank: 'blank',
      Custom: 'custom',
    }
    const backendCategory = categoryMap[generateCategory.value] || 'custom'
    await api.generateTemplateFromProject(generateProjectId.value, generateTemplateName.value.trim(), backendCategory)
    toast.success(t('templates.generateSuccess') || '模板生成成功')
    showGenerateFromProjectDialog.value = false
    generateProjectId.value = ''
    generateTemplateName.value = ''
    generateCategory.value = 'Custom'
    await loadTemplates()
  } catch (e: any) {
    toast.error(`${t('templates.generateFailed') || '生成失败'}: ${e?.toString() || e}`)
  } finally {
    isGenerating.value = false
  }
}

const progressPercent = computed(() => {
  if (!createProgress.value) return 0
  return Math.round(createProgress.value.progress * 100)
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="shrink-0 px-6 pt-6 pb-4">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">{{ t('templates.title') }}</h1>
          <p class="text-sm text-gray-500 dark:text-content-muted mt-1">{{ t('templates.subtitle') }}</p>
        </div>
        <div class="flex items-center gap-2">
          <button
            @click="showGenerateFromProjectDialog = true"
            class="px-4 py-2 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
          >
            {{ t('templates.generateFromProject') || '从项目生成' }}
          </button>
          <button
            @click="showImportDialog = true"
            class="px-4 py-2 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors"
          >
            {{ t('templates.importUrl') }}
          </button>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="flex-1 relative">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('projects.search') || 'Search...'"
            class="w-full pl-10 pr-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 focus:border-transparent outline-none"
          />
        </div>
        <div class="flex gap-1 flex-wrap">
          <button
            v-for="cat in categories"
            :key="cat"
            @click="categoryFilter = cat"
            :class="[
              'px-3 py-1.5 text-xs font-medium rounded-full transition-colors',
              categoryFilter === cat
                ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300'
                : 'bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-secondary hover:bg-gray-200 dark:hover:bg-surface-border'
            ]"
          >
            {{ categoryIcon(cat) }} {{ cat === 'all' ? t('templates.category.all') : t(`templates.category.${cat}`) }}
          </button>
        </div>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-6 pb-6">
      <SkeletonList v-if="isLoading" :count="4" />

      <div v-else-if="loadError" class="text-center py-12">
        <svg class="w-12 h-12 mx-auto text-red-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        <p class="text-red-500 text-sm mb-2">{{ loadError }}</p>
        <button
          class="px-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700 bg-primary-50 hover:bg-primary-100 dark:bg-primary-900/20 dark:text-primary-400 dark:hover:bg-primary-900/30 rounded-lg transition-colors"
          @click="loadTemplates"
        >
          {{ t('common.retry') || '重试' }}
        </button>
      </div>

      <EmptyState
        v-else-if="filteredTemplates.length === 0"
        :title="t('templates.empty')"
        :description="t('templates.emptyDesc')"
        icon="template"
        :shortcuts="[
          { key: 'Ctrl+K', description: t('sidebar.openCommandPaletteShortcut') },
        ]"
      />

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="tpl in filteredTemplates"
          :key="tpl.template_id"
          class="group relative bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border hover:border-primary-300 dark:hover:border-primary-600 hover:shadow-lg transition-all duration-200 cursor-pointer overflow-hidden"
          @click="openDetail(tpl)"
          @contextmenu="onTemplateContextMenu($event, tpl)"
        >
          <div class="p-5">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-primary-50 dark:bg-primary-900/20 flex items-center justify-center text-xl">
                  {{ categoryIcon(tpl.category) }}
                </div>
                <div>
                  <h3 class="font-semibold text-gray-900 dark:text-content-primary text-sm">{{ tpl.name }}</h3>
                  <p class="text-xs text-gray-500 dark:text-content-muted">{{ t(`templates.category.${tpl.category}`) }}</p>
                </div>
              </div>
              <span
                v-if="tpl.is_builtin"
                class="px-2 py-0.5 text-xs font-medium rounded-full bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400"
              >
                {{ t('templates.builtin') }}
              </span>
            </div>

            <p class="text-xs text-gray-600 dark:text-content-secondary line-clamp-2 mb-3">{{ tpl.description }}</p>

            <div class="flex items-center gap-3 text-xs text-gray-500 dark:text-content-muted mb-4">
              <span class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                {{ tpl.godot.version }}
              </span>
              <span v-if="tpl.plugins.length > 0" class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                </svg>
                {{ tpl.plugins.length }} {{ t('templates.plugins') }}
              </span>
              <span class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
                {{ tpl.directories.length }} {{ t('templates.directories') }}
              </span>
            </div>

            <div class="flex flex-wrap gap-1.5 mb-4">
              <span
                v-for="tag in tpl.tags.slice(0, 4)"
                :key="tag"
                class="px-2 py-0.5 text-xs rounded-full bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-secondary"
              >
                {{ tag }}
              </span>
            </div>

            <button
              @click.stop="openCreateDialog(tpl)"
              class="w-full py-2 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
            >
              {{ t('templates.createProject') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail Dialog -->
    <Teleport to="body">
      <div v-if="showDetailDialog && selectedTemplate" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="showDetailDialog = false"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto">
          <div class="p-6">
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-3">
                <div class="w-12 h-12 rounded-xl bg-primary-50 dark:bg-primary-900/20 flex items-center justify-center text-2xl">
                  {{ categoryIcon(selectedTemplate.category) }}
                </div>
                <div>
                  <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary">{{ selectedTemplate.name }}</h2>
                  <p class="text-sm text-gray-500 dark:text-content-muted">
                    {{ t(`templates.category.${selectedTemplate.category}`) }}
                    <span v-if="selectedTemplate.author"> · {{ selectedTemplate.author }}</span>
                  </p>
                </div>
              </div>
              <button @click="showDetailDialog = false" class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-500">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <p class="text-sm text-gray-600 dark:text-content-secondary mb-5">{{ selectedTemplate.description }}</p>

            <div v-if="selectedTemplate.preview_images && selectedTemplate.preview_images.length > 0" class="mb-5">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.previewImages') || '预览' }}</h3>
              <div class="grid grid-cols-2 gap-2">
                <img
                  v-for="(img, idx) in selectedTemplate.preview_images"
                  :key="idx"
                  :src="img"
                  :alt="`Preview ${idx + 1}`"
                  class="w-full h-32 object-cover rounded-lg border border-gray-200 dark:border-surface-border"
                  @error="($event.target as HTMLImageElement).style.display = 'none'"
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4 mb-5">
              <div class="p-3 rounded-lg bg-gray-50 dark:bg-surface-layer">
                <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.godotVersion') }}</p>
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ selectedTemplate.godot.version }}{{ selectedTemplate.godot.mono ? ' (Mono)' : '' }}</p>
              </div>
              <div class="p-3 rounded-lg bg-gray-50 dark:bg-surface-layer">
                <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.plugins') }}</p>
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ selectedTemplate.plugins.length }}</p>
              </div>
            </div>

            <div v-if="selectedTemplate.plugins.length > 0" class="mb-5">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.plugins') }}</h3>
              <div class="space-y-2">
                <div
                  v-for="plugin in selectedTemplate.plugins"
                  :key="plugin.name"
                  class="flex items-center justify-between p-2.5 rounded-lg bg-gray-50 dark:bg-surface-layer"
                >
                  <div>
                    <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ plugin.name }}</p>
                    <p class="text-xs text-gray-500 dark:text-content-muted">v{{ plugin.version }} · {{ plugin.source }}</p>
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <input
                  id="mobile-support"
                  v-model="enableMobileSupport"
                  type="checkbox"
                  :disabled="isCreating"
                  class="w-4 h-4 rounded border-gray-300 dark:border-surface-border text-primary-600 focus:ring-primary-500"
                />
                <label for="mobile-support" class="text-sm text-gray-700 dark:text-content-secondary cursor-pointer">
                  {{ t('templates.enableMobileSupport') || '添加移动端支持（触摸控件 + 虚拟摇杆）' }}
                </label>
              </div>
            </div>

            <div v-if="selectedTemplate.directories.length > 0" class="mb-5">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.directories') }}</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="dir in selectedTemplate.directories"
                  :key="dir.path"
                  class="px-2.5 py-1 text-xs rounded-lg bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary font-mono"
                >
                  {{ dir.path }}
                </span>
              </div>
            </div>

            <div v-if="selectedTemplate.export_presets.length > 0" class="mb-5">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.exportPresets') }}</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="preset in selectedTemplate.export_presets"
                  :key="preset.name"
                  class="px-2.5 py-1 text-xs rounded-lg bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary"
                >
                  {{ preset.name }} ({{ preset.platform }})
                </span>
              </div>
            </div>

            <div class="flex gap-3">
              <button
                @click="showDetailDialog = false; openCreateDialog(selectedTemplate!)"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors"
              >
                {{ t('templates.createProject') }}
              </button>
              <button
                v-if="!selectedTemplate.is_builtin"
                @click="deleteTargetId = selectedTemplate.template_id; showDeleteConfirm = true"
                class="px-4 py-2.5 text-sm font-medium rounded-lg border border-red-300 dark:border-red-800 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
              >
                {{ t('common.delete') || 'Delete' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Create Project Dialog -->
    <Teleport to="body">
      <div v-if="showCreateDialog && selectedTemplate" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isCreating && (showCreateDialog = false)"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-md w-full mx-4">
          <div class="p-6">
            <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary mb-4">
              {{ t('templates.createProject') }} — {{ selectedTemplate.name }}
            </h2>

            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.projectName') || 'Project Name' }}</label>
                <input
                  v-model="createProjectName"
                  type="text"
                  :disabled="isCreating"
                  @input="validateProjectName"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                  :class="{ 'border-red-400 dark:border-red-500': projectNameError }"
                />
                <p v-if="projectNameError" class="mt-1 text-xs text-red-500">{{ projectNameError }}</p>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.targetDir') || 'Target Directory' }}</label>
                <div class="flex gap-2">
                  <input
                    v-model="createTargetDir"
                    type="text"
                    :disabled="isCreating"
                    class="flex-1 px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                  />
                  <button
                    @click="selectTargetDir"
                    :disabled="isCreating"
                    class="px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer disabled:opacity-50"
                  >
                    ...
                  </button>
                </div>
              </div>
            </div>

            <div v-if="createProgress" class="mt-4">
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs text-gray-500 dark:text-content-muted">{{ createProgress.message }}</span>
                <span class="text-xs font-medium text-primary-600 dark:text-primary-400">{{ progressPercent }}%</span>
              </div>
              <div class="w-full bg-gray-200 dark:bg-surface-border rounded-full h-1.5">
                <div
                  class="bg-primary-600 h-1.5 rounded-full transition-all duration-300"
                  :style="{ width: `${progressPercent}%` }"
                ></div>
              </div>
            </div>

            <div class="flex gap-3 mt-6">
              <button
                @click="showCreateDialog = false"
                :disabled="isCreating"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') || 'Cancel' }}
              </button>
              <button
                @click="handleCreate"
                :disabled="isCreating || !isValidProjectName || !createTargetDir.trim()"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
              >
                {{ isCreating ? t('templates.creating') : t('templates.createProject') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Created Success Quick Access -->
    <Teleport to="body">
      <div v-if="lastCreatedProjectId" class="fixed bottom-6 right-6 z-50 animate-fade-in">
        <div class="bg-green-600 text-white rounded-xl shadow-lg px-4 py-3 flex items-center gap-3">
          <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm font-medium">{{ t('templates.createSuccess') }}</span>
          <button
            v-if="lastCreatedProject"
            @click="openProjectWithEngine(lastCreatedProject!)"
            class="ml-1 px-2.5 py-1 text-xs font-medium bg-white/20 hover:bg-white/30 rounded-lg transition-colors"
          >
            {{ t('projects.openWithEngine') }}
          </button>
          <button
            v-if="lastCreatedProject"
            @click="openInFileManager(lastCreatedProject!.path)"
            class="px-2.5 py-1 text-xs font-medium bg-white/20 hover:bg-white/30 rounded-lg transition-colors"
          >
            {{ t('projects.openInFileManager') }}
          </button>
          <button
            @click="lastCreatedProjectId = ''; lastCreatedProject = null"
            class="ml-1 text-green-200 hover:text-white transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </Teleport>

    <!-- Import URL Dialog -->
    <Teleport to="body">
      <div v-if="showImportDialog" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isImporting && (showImportDialog = false)"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-md w-full mx-4">
          <div class="p-6">
            <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary mb-4">{{ t('templates.importUrl') }}</h2>
            <input
              v-model="importUrl"
              type="url"
              :placeholder="t('templates.importUrlPlaceholder')"
              :disabled="isImporting"
              class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
            />
            <div class="flex gap-3 mt-6">
              <button
                @click="showImportDialog = false"
                :disabled="isImporting"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') || 'Cancel' }}
              </button>
              <button
                @click="handleImport"
                :disabled="isImporting || !importUrl.trim() || !isOnline"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
              >
                {{ isImporting ? '...' : !isOnline ? (t('common.offlineImportTip') || '离线无法导入') : (t('common.import') || '导入') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Generate From Project Dialog -->
    <Teleport to="body">
      <div v-if="showGenerateFromProjectDialog" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isGenerating && (showGenerateFromProjectDialog = false)"></div>
        <div class="relative bg-white dark:bg-surface-card rounded-2xl shadow-2xl max-w-md w-full mx-4">
          <div class="p-6">
            <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary mb-4">{{ t('templates.generateFromProject') || '从项目生成模板' }}</h2>
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.selectProject') || '选择项目' }}</label>
                <ProjectSelector v-model="generateProjectId" :projects="projects" :placeholder="t('templates.selectProjectPlaceholder') || '请选择项目'" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.templateName') || '模板名称' }}</label>
                <input
                  v-model="generateTemplateName"
                  type="text"
                  :placeholder="t('templates.templateNamePlaceholder') || '输入模板名称'"
                  :disabled="isGenerating"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.category') || '分类' }}</label>
                <select
                  v-model="generateCategory"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none"
                >
                  <option value="Custom">{{ t('templates.category.Custom') || '自定义' }}</option>
                  <option value="Starter2D">{{ t('templates.category.Starter2D') || '2D入门' }}</option>
                  <option value="Starter3D">{{ t('templates.category.Starter3D') || '3D入门' }}</option>
                  <option value="RPG">{{ t('templates.category.RPG') || 'RPG' }}</option>
                  <option value="Platformer">{{ t('templates.category.Platformer') || '平台跳跃' }}</option>
                  <option value="Multiplayer">{{ t('templates.category.Multiplayer') || '多人游戏' }}</option>
                  <option value="Mobile">{{ t('templates.category.Mobile') || '移动端' }}</option>
                </select>
              </div>
            </div>
            <div class="flex gap-3 mt-6">
              <button
                @click="showGenerateFromProjectDialog = false"
                :disabled="isGenerating"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') || '取消' }}
              </button>
              <button
                @click="handleGenerateFromProject"
                :disabled="isGenerating || !generateProjectId || !generateTemplateName.trim()"
                class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
              >
                {{ isGenerating ? '...' : (t('common.generate') || '生成') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div v-if="showContextMenu" class="fixed inset-0 z-50" @click="closeContextMenu" @contextmenu.prevent="closeContextMenu">
        <div
          class="fixed bg-white dark:bg-surface-card rounded-lg shadow-xl border border-gray-200 dark:border-surface-border py-1.5 min-w-[180px] z-50"
          :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
          @click.stop
        >
          <button
            @click="openCreateDialog(contextMenuTemplate!); closeContextMenu()"
            class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-content-secondary hover:bg-gray-100 dark:hover:bg-surface-hover flex items-center gap-2.5"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
            {{ t('templates.createProject') }}
          </button>
          <button
            @click="openDetail(contextMenuTemplate!); closeContextMenu()"
            class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-content-secondary hover:bg-gray-100 dark:hover:bg-surface-hover flex items-center gap-2.5"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
            {{ t('templates.viewDetail') || t('common.viewDetail') }}
          </button>
          <button
            v-if="contextMenuTemplate && !contextMenuTemplate.is_builtin"
            @click="deleteTargetId = contextMenuTemplate!.template_id; showDeleteConfirm = true; closeContextMenu()"
            class="w-full px-4 py-2 text-left text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/10 flex items-center gap-2.5"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
            {{ t('common.delete') }}
          </button>
        </div>
      </div>
    </Teleport>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('templates.deleteConfirm')"
      :confirm-text="t('common.delete')"
      confirm-color="red"
      @confirm="handleDelete"
      @update:model-value="(v: boolean) => { if (!v) deleteTargetId = '' }"
    />

    <!-- 引擎选择对话框 -->
    <Teleport to="body">
      <div v-if="showEngineSelectDialog && engineSelectProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="closeEngineSelectDialog">
        <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-md shadow-xl max-h-[80vh] flex flex-col" @click.stop>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-1">{{ t('projects.openWithEngine') }}</h3>
          <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
            {{ t('projects.openWithEngineDesc') }}
            <span class="font-mono text-xs bg-gray-100 dark:bg-surface-hover px-1.5 py-0.5 rounded ml-1">Godot {{ engineSelectProject.godot_version }}</span>
          </p>
          <div v-if="isLoadingEngines" class="flex-1 flex items-center justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
          </div>
          <div v-else-if="matchedEngines.length === 0" class="flex-1 py-8 text-center">
            <svg class="mx-auto h-10 w-10 text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            <p class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('projects.noMatchingEngines') }}</p>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('projects.noMatchingEnginesDesc') }}</p>
          </div>
          <div v-else class="flex-1 overflow-y-auto space-y-2 min-h-0">
            <button
              v-for="me in matchedEngines"
              :key="me.engine.engine_id"
              @click="launchWithEngine(me.engine.engine_id)"
              :disabled="isLaunching"
              :class="[
                'w-full text-left p-3 rounded-lg border transition-colors disabled:opacity-40 disabled:cursor-not-allowed',
                me.engine.engine_id === engineSelectProject?.last_used_engine_id
                  ? 'border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-900/10'
                  : 'border-gray-200 dark:border-surface-border hover:border-primary-300 dark:hover:border-primary-700 hover:bg-primary-50 dark:hover:bg-primary-900/10'
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1.5">
                    {{ me.engine.name }}
                    <span v-if="me.engine.engine_id === engineSelectProject?.last_used_engine_id" class="text-xs text-primary-600 dark:text-primary-400 font-normal">{{ t('projects.lastUsedEngine') }}</span>
                  </div>
                  <div class="text-xs text-gray-500 dark:text-content-muted mt-0.5 font-mono flex items-center gap-1.5">v{{ me.engine.version }}<span v-if="me.engine.is_mono" class="text-[10px] px-1 py-0.5 rounded bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400 font-sans font-medium">{{ t('projects.monoLabel') }}</span></div>
                </div>
                <span :class="['text-xs px-2 py-0.5 rounded-full font-medium ml-2 flex-shrink-0', getMatchLevelClass(me.match_level)]" :title="getMatchLevelDesc(me.match_level)">{{ getMatchLevelLabel(me.match_level) }}</span>
              </div>
            </button>
          </div>
          <div class="flex justify-end mt-4 pt-3 border-t border-gray-200 dark:border-surface-border">
            <button @click="closeEngineSelectDialog" class="btn-secondary">{{ t('common.cancel') }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
